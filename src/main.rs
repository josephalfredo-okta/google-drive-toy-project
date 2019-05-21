extern crate hyper;
extern crate hyper_tls;

use bytes::Bytes;
use hyper::rt::{self, Future, Stream};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::io::{self, Write};
use std::sync::Arc;
use std::env;
use std::process;

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 3{
    println!("usage: google-drive-fs [file-id] [token]");
    process::exit(1);
  }
  let file_name = args[1].clone();
  let token = format!("Bearer {}", args[2]);
  let token = Arc::new(token);

  println!("Starting program - calling rt::run");
  rt::run(rt::lazy(move || {
    download_file(token.clone(), file_name)
      .and_then(|bytes| upload_file(token, bytes))
      .map(|_| {
        println!("finished the future chain..");
      })
      .map_err(|err| {
        eprintln!("runtime failed, err={}", err);
      })
  }))
}

fn download_file(token: Arc<String>, file_name: String) -> impl Future<Item = Bytes, Error = hyper::Error> {
  let https = HttpsConnector::new(4).expect("TLS initialization failed");
  let client = Client::builder().build::<_, hyper::Body>(https);
  let url = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file_name);
  let download_req = Request::get(url)
  .header("Content-Type", "application/json")
  .header("Authorization", token.to_string())
  .body(Body::empty())
  .expect("request builder error");

  client
    .request(download_req)
    .and_then(|res| {
      println!("Download Response: {}", res.status());

      res.into_body().concat2()
    })
    .map(|body| {
      let bytes = Bytes::from(body);
      println!("Downloaded file of size = {}", bytes.len());
      bytes
    })
}

fn upload_file(
  token: Arc<String>,
  file_bytes: Bytes,
) -> impl Future<Item = (), Error = hyper::Error> {
  println!("Uploading file of size = {}", file_bytes.len());

  let https = HttpsConnector::new(4).expect("TLS initialization failed");
  let client = Client::builder().build::<_, hyper::Body>(https);
  let upload_req =
    Request::post("https://www.googleapis.com/upload/drive/v3/files?uploadType=media")
      .header("Content-Type", "image/jpg")
      .header("Authorization", token.to_string())
      .header("Content-Length", file_bytes.len())
      .header("Accept", "application/json")
      .body(Body::from(file_bytes))
      .expect("request builder error");

  client.request(upload_req).and_then(|res| {
    println!("Upload Response: {}", res.status());
    res.into_body().for_each(|chunk| {
      io::stdout()
        .write_all(&chunk)
        .map_err(|e| panic!("example expects stdout is open, error={}", e))
    })
  }).map(|_|{})
}
