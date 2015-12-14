extern crate zensend;

use std::env;
use zensend::Client;


fn main() {
  let args : Vec<_> = env::args().collect();


  let client = Client::new(args[1].clone());
  
  let res = client.get_prices();
  print!("{:?}", res);
}