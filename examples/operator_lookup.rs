extern crate zensend;

use std::env;
use zensend::Client;


fn main() {
  let args : Vec<_> = env::args().collect();


  let client = Client::new(args[1].clone());
  
  let res = client.lookup_operator(args[2].as_ref());
  print!("{:?}", res);
}