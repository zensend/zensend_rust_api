extern crate zensend;

use std::env;
use zensend::Client;
use zensend::Message;
use zensend::SmsEncoding;
use zensend::OriginatorType;

fn main() {
  let args : Vec<_> = env::args().collect();


  let client = Client::new(args[1].clone());
  
  let res = client.create_sub_account(args[2].clone());
  print!("{:?}", res);
}
