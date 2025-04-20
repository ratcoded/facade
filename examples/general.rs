use tokio_tungstenite::tungstenite::Message;
use tokio::time::{Duration, timeout, sleep};
use std::time::{SystemTime, UNIX_EPOCH};
use futures_util::StreamExt;
use facade::WindowData;
use facade::Packet;
use facade::Client;
use facade::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Error> {
   let mut instance = Client::new();
   // get our required data //
   let ( session_key, static_key,
      csrf_token, six_key, topic
   ) = instance.fetch_required().await?;
   println!("[debug] scraping results:");
   println!("csrf_token: {:?}", &csrf_token);
   println!("static_key: {:?}", &static_key);
   println!("session_key: {:?}", &session_key);
   println!("six_key: {:?}", &six_key);
   println!("topic: {:?}", &topic);
   println!("[debug] ---------------- ");
   // connect and start readloop immediately //
   instance.connect(&csrf_token, &six_key).await?;
   if let Some(channel) = &instance.read {
      let channel = Arc::clone(channel);
      let _readloop = tokio::spawn(async move {
         let mut read = channel.lock().await;
         loop {
            match timeout(Duration::from_secs(31), read.next()).await {
               Ok(Some(Ok(Message::Text(text)))) => { println!("[info] received response: {}", text) }
               Ok(Some(_)) => { println!("[warn] received non-text message (ignored)") }
               Ok(None) => { println!("[error] connection abruptly ended"); break; }
               Err(_) => { println!("[error] no response from server in 30s") }
            }
         }
      });
   }
   // send our initial packets (phx_join, heartbeat) //
   instance.send(
      &Packet::phx_join(
         &session_key,
         &static_key,
         &csrf_token,
         &topic,
         &instance.css,
         &instance.js,
         &instance.https,
      )
   ).await?;
   instance.send(&Packet::heartbeat(10)).await?;
   // send a password change & puzzle info (move floors) //
   instance.send(&Packet::password_change(11, &topic, "rHQHHtfhbZo")).await?;
   instance.send(
      &Packet::puzzle_info(
         4, 12, &topic,
         SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap().as_millis(),
         WindowData {
            screen_left: 666, screen_top: 666,
            inner_width: 666, inner_height: 666,
            outer_width: 666, outer_height: 666
         }
      )
   ).await?;
   // keep program alive to keep readloop going //
   sleep(Duration::from_secs(61)).await;
   Ok(())
}
