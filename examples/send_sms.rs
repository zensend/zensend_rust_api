extern crate zensend;

use std::env;
use zensend::Client;
use zensend::Message;
use zensend::SmsEncoding;
use zensend::OriginatorType;

fn main() {
  let args : Vec<_> = env::args().collect();


  let client = Client::new(args[1].clone());
  
  let res = client.send_sms(Message { originator: "ZenSend", body: "Hello 😱", originator_type: OriginatorType::Alpha, sms_encoding: SmsEncoding::Ucs2, numbers: &[args[2].as_ref()], ..Default::default()});
  print!("{:?}", res);
}
