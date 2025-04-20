use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream, tungstenite::{handshake::client::generate_key, http, Message}};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use tokio::{sync::Mutex, net::TcpStream};
use scraper::{Html, Selector};
use crate::packet::Packet;
use std::sync::Arc;
use crate::Error;
use core::panic;

#[derive(Debug)]
pub struct Client {
   // config //
   pub https: String,
   pub host: String,
   pub css: String,
   pub ws: String,
   pub js: String,
   // channels //
   write: Option<Arc<tokio::sync::Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>,
   pub read: Option<Arc<tokio::sync::Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>>,
}

impl Client {
   /// Constructs a new Client using default configuration
   pub fn new() -> Self {
      let host = String::from("sixfaces.top");
      let https = format!("https://{}/", host);
      let ws = format!("wss://{}/live/websocket", host);
      let css = format!("{}/assets/app-5bbae6cfa1b3c6bc7345a903f92f3724.css?vsn=d", https);
      let js = format!("{}/assets/app-c882a216391dd10a1bad0169d2f0ce71.js?vsn=d", https);
      Self {
         https, host,
         ws, css, js,
         write: None,
         read: None
      }
   }

   pub async fn fetch_required(&self) -> Result<(String, String, String, String, String), Error> {
      let [mut session_key, mut static_key, mut csrf_token, mut topic] = [const {String::new()}; 4];
      let six_key; // for some reason the compiler dislikes it when i place this up there and throwns a warning which triggers my ocd brain
      let response = reqwest::Client::new().get(&self.https)
         .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0")
         .header("Accept-Language", "en-US,en;q=0.5")
         .header("Cookie", "_cookie_consent=true;")
         .header("Connection", "keep-alive")
         .header("Host", &self.host)
         .header("Accept", "*/*")
         .send().await?;
      // obtain six_key //
      let Some(header) = response.headers().get("set-cookie") else { panic!("<ERROR>") };
      let Ok(header) = header.to_str() else { panic!("<ERROR>") };
      let Some(start) = header.find("_six_key=") else { panic!("<ERROR>") };
      let part = &header[start+9..];
      if let Some(end) = part.find(';') {
         six_key = part[..end].to_string();
      } else { six_key = part.to_string(); }
      // obtain csrf token //
      let document = Html::parse_document(&response.text().await?);
      let selector = Selector::parse(r#"meta[name="csrf-token"]"#).unwrap();
      if let Some(target) = document.select(&selector).next() {
         csrf_token = target.value()
            .attr("content")
            .map(String::from)
            .unwrap()}
      // obtain session key, static key & topic //
      let selector = Selector::parse("[data-phx-main]").unwrap();
      if let Some(target) = document.select(&selector).next() {
         session_key = target.value()
            .attr("data-phx-session")
            .map(String::from)
            .unwrap();
         static_key = target.value()
            .attr("data-phx-static")
            .map(String::from)
            .unwrap();
         topic = target.value().id()
            .map(|id| id.replace("phx-", "lv:phx-"))
            .unwrap()}
      // return all of it //
      Ok((session_key, static_key, csrf_token, six_key, topic))
   }

   pub async fn connect(&mut self, csrf_token: &String, six_key: &String) -> Result<(), Error> {
      let mut endpoint = url::Url::parse(&self.ws).unwrap();
      endpoint.query_pairs_mut()
         .append_pair("_csrf_token", csrf_token)
         .append_pair("_track_static[0]", &self.css)
         .append_pair("_track_static[1]", &self.js)
         .append_pair("_mounts", "0")
         .append_pair("_live_referer", &self.https)
         .append_pair("vsn", "2.0.0");
      let request = http::Request::builder()
         .method("GET").uri(endpoint.as_str())
         .header("Accept", "*/*")
         .header("Accept-Language", "en-US,en;q=0.5")
         .header("Cache-Control", "no-cache")
         .header("Connection", "keep-alive, Upgrade")
         .header("Cookie", format!("_cookie_consent=true; _six_key={}", six_key))
         .header("Host", &self.host)
         .header("Origin", &self.https)
         .header("Sec-Fetch-Dest", "empty")
         .header("Sec-Fetch-Mode", "websocket")
         .header("Sec-Fetch-Site", "same-origin")
         .header("Sec-Websocket-Key", generate_key())
         .header("Sec-Websocket-Extensions", "permessage-deflate")
         .header("Sec-Websocket-Version", "13")
         .header("Upgrade", "websocket")
         .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0")
         .body(()).unwrap();
      // make request -> extract read/write channels //
      let (stream, _) = connect_async(request).await?;
      let (proc, read) = stream.split();
      (self.write, self.read) = (
         Some(Arc::new(Mutex::new(proc))),
         Some(Arc::new(Mutex::new(read)))
      ); Ok(())
   }

   pub async fn send(&mut self, raw: &Packet) -> Result<(), Error> {
      let packet = raw.parse().unwrap();
      println!("[info] sending packet: {:?}", raw); // log send
      let Some(channel) = &self.write else { return Ok(()) };
      Ok(Arc::clone(&channel).lock().await.send(packet).await?)
   }
}