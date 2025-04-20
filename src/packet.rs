use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::WindowData;

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet(
   #[serde(default)] pub Option<String>,                 // join id //
   #[serde(default)] pub Option<String>,                 // ref id  //
   #[serde(default)] pub Option<String>,                 // topic   //
   #[serde(default)] pub Option<String>,                 // event   //
   #[serde(default)] pub Option<HashMap<String, Value>>, // payload //
);

impl Packet {
   // unfinished
   pub fn heartbeat(ref_id: u32) -> Self {
      Self (
         None,
         Some(ref_id.to_string()),
         Some(String::from("phoenix")),
         Some(String::from("heartbeat")),
         Some(HashMap::new())
      )
   }

   pub fn phx_join(session_key: &String, static_key: &String, csrf_token: &String, topic: &String, css: &String, js: &String, https: &String) -> Self {
      Self (
         Some(4.to_string()),
         Some(4.to_string()),
         Some(topic.to_string()),
         Some(String::from("phx_join")),
         Some({
            let mut params: HashMap<String, Value> = HashMap::with_capacity(3);
            params.insert(String::from("_csrf_token"), json!(csrf_token));
            params.insert(String::from("_track_static"), json!(vec![css, js]));
            params.insert(String::from("_mounts"), json!(0));
            let mut payload: HashMap<String, Value> = HashMap::with_capacity(4);
            payload.insert(String::from("url"), json!(https));
            payload.insert(String::from("params"), json!(params));
            payload.insert(String::from("session"), json!(session_key));
            payload.insert(String::from("static"), json!(static_key));
            payload
         })
      )
   }

   pub fn password_change(ref_id: u32, topic: &String, password: &str) -> Self {
      Self (
         Some(4.to_string()),
         Some(ref_id.to_string()),
         Some(topic.to_string()),
         Some(String::from("event")),
         Some({
            let mut payload: HashMap<String, Value> = HashMap::with_capacity(4);
            payload.insert(String::from("type"), json!("form"));
            payload.insert(String::from("event"), json!("password_change"));
            payload.insert(String::from("value"), json!(format!("password={}&_target=password", password)));
            let uploads: HashMap<String, Value> = HashMap::new();
            payload.insert(String::from("uploads"), json!(uploads));
            payload
         })
      )
   }

   pub fn puzzle_info(floor_id: u32, ref_id: u32, topic: &String, connect_time: u128, window: WindowData) -> Self {
      Self (
         Some(4.to_string()),
         Some(ref_id.to_string()),
         Some(topic.to_string()),
         Some(String::from("event")),
         Some({
            let mut payload: HashMap<String, Value> = HashMap::with_capacity(3);
            payload.insert(String::from("type"), json!("hook"));
            payload.insert(String::from("event"), json!("puzzle-info"));
            let mut value: HashMap<String, Value> = HashMap::with_capacity(9);
            value.insert(String::from("connectTime"), json!(connect_time));
            value.insert(String::from("floorId"), json!(format!("floor_{}", floor_id)));
            value.insert(String::from("screenLeft"), json!(window.screen_left));
            value.insert(String::from("screenTop"), json!(window.screen_top));
            value.insert(String::from("innerWidth"), json!(window.inner_width));
            value.insert(String::from("innerHeight"), json!(window.inner_height));
            value.insert(String::from("outerWidth"), json!(window.outer_width));
            value.insert(String::from("outerHeight"), json!(window.outer_height));
            let puzzle_info: HashMap<String, Value> = HashMap::new();
            value.insert(String::from("puzzleInfo"), json!(puzzle_info));
            payload.insert(String::from("value"), json!(value));
            payload
         })
      )
   }

   // Packet -> Message //
   pub fn parse(&self) -> Result<Message, serde_json::Error> {
      Ok(Message::text(serde_json::to_string(&self)?))
   }
}

// Message -> Packet //
impl TryFrom<Message> for Packet {
   type Error = serde_json::Error;
   fn try_from(data: Message) -> Result<Self, Self::Error> {
      Ok(serde_json::from_str::<Packet>(
         data.to_text().unwrap()
      )?)
   }
}

// Utf8Bytes -> Message -> Packet
impl TryFrom<Utf8Bytes> for Packet {
   type Error = serde_json::Error;
   fn try_from(data: Utf8Bytes) -> Result<Self, Self::Error> {
      Ok(serde_json::from_str::<Packet>(
         Message::text(data).to_text().unwrap()
      )?)
   }
}