//!### HTTP
//!
//!```rust,no_run
//!use gloo_net::http::Request;
//!async fn run() -> Result<(), gloo_net::Error> {
//!    let resp = Request::get("/path")
//!        .send()
//!        .await
//!        .unwrap();
//!    assert_eq!(resp.status(), 200);
//!    Ok(())
//!}
//!```
//!with body
//!```rust,no_run
//!use gloo_net::http::Request;
//!use serde::Serialize;
//!#[derive(Serialize)]
//!struct Post<'a> {
//!    title: &'a str,
//!    body: &'a str,
//!    #[serde(rename = "userId")]
//!    user_id: u32,
//!}
//!
//!async fn run() -> Result<(), gloo_net::Error> {
//!    let response = Request::post("https://example.com/posts")
//!        .json(&Post { title: "hello", body: "world", user_id: 1 })?
//!        .send()
//!        .await?;
//!    let data = response.text().await?;
//!    Ok(())
//!}
//!```
//!
//!
//!### WebSocket
//!
//!```rust,no_run
//!use gloo_net::websocket::{Message, futures::WebSocket};
//!use wasm_bindgen_futures::spawn_local;
//!use futures::{SinkExt, StreamExt};
//!let mut ws = WebSocket::open("wss://echo.websocket.org").unwrap();
//!let (mut write, mut read) = ws.split();
//!
//!spawn_local(async move {
//!    write.send(Message::Text(String::from("test"))).await.unwrap();
//!    write.send(Message::Text(String::from("test 2"))).await.unwrap();
//!});
//!
//!spawn_local(async move {
//!    while let Some(msg) = read.next().await {
//!        gloo_console::log!(format!("1. {:?}", msg));
//!    }
//!    gloo_console::log!("WebSocket Closed");
//!})
//!```
//!
//!### EventSource
//!
//!```rust,no_run
//!use gloo_net::eventsource::futures::EventSource;
//!use wasm_bindgen_futures::spawn_local;
//!use futures::{stream, StreamExt};
//!let mut es = EventSource::new("http://api.example.com/ssedemo.php").unwrap();
//!let stream_1 = es.subscribe("some-event-type").unwrap();
//!let stream_2 = es.subscribe("another-event-type").unwrap();
//!
//!spawn_local(async move {
//!    let mut all_streams = stream::select(stream_1, stream_2);
//!    while let Some(Ok((event_type, msg))) = all_streams.next().await {
//!        gloo_console::log!(format!("1. {event_type}: {msg:?}"))
//!    }
//!    gloo_console::log!("EventSource Closed");
//!})
//!```

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
#[cfg(feature = "eventsource")]
#[cfg_attr(docsrs, doc(cfg(feature = "eventsource")))]
pub mod eventsource;
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub mod http;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod websocket;

pub use error::*;
