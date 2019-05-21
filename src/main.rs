extern crate hyper;
extern crate hyper_tls;

use bytes::Bytes;
use hyper::rt::{self, Future, Stream};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::io::{self, Write};
use std::sync::Arc;

fn main() {
  let token = Arc::new("Bearer ya29.GlsQB_MtY-m6rTIiE5p12okrXsfQI65jd4Mu3PlWu9F2Q7mnsRWAH0vo4H4nuvMn1lcA1s7orlEEb_0A7h5T1uv7YkeFiNfYCPg0TS0JgBoxj1HRXXIjdVu9qaLt");

  println!("Starting program - calling rt::run");
  rt::run(rt::lazy(|| {
    download_file(token.clone())
      .and_then(|bytes| upload_file(token, bytes))
      .map(|_| {
        println!("finished the future chain..");
      })
      .map_err(|err| {
        eprintln!("runtime failed, err={}", err);
      })
  }))
}

fn download_file(token: Arc<&str>) -> impl Future<Item = Bytes, Error = hyper::Error> {
  let https = HttpsConnector::new(4).expect("TLS initialization failed");
  let client = Client::builder().build::<_, hyper::Body>(https);
  let download_req = Request::get(
    "https://www.googleapis.com/drive/v3/files/1nNjZyZdF0WtzCUcKZE1UwF1sWHaheY5H?alt=media",
  )
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
  token: Arc<&str>,
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
