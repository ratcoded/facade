// expose //
pub use crate::client::Client;
pub use crate::packet::Packet;
pub use crate::errors::Error;
// import //
mod errors;
mod packet;
mod client;
pub struct WindowData {
   pub screen_left: usize,
   pub screen_top: usize,
   pub inner_width: usize,
   pub inner_height: usize,
   pub outer_width: usize,
   pub outer_height: usize
}