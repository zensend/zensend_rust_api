extern crate rustc_serialize;
extern crate hyper;
extern crate url;

use rustc_serialize::json;
use hyper::header::{Headers, ContentType};
use hyper::status::StatusCode;
use hyper::client::Response;
use url::form_urlencoded;
use hyper::mime::{Mime, TopLevel, SubLevel};
use std::io::Read;
use std::collections::HashMap;

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct SmsResult {
  txguid: String,
  numbers: i32,
  smsparts: i32,
  encoding: String,
  cost_in_pence: f64,
  new_balance_in_pence: f64
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct Prices {
  prices_in_pence: HashMap<String, f64>
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct Balance {
  balance: f64
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct OperatorLookupResult {
  mcc: String,
  mnc: String,
  operator: String,
  cost_in_pence: f64,
  new_balance_in_pence: f64
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct ZenSendError {
  failcode: String,
  parameter: String,
  cost_in_pence: Option<f64>,
  new_balance_in_pence: Option<f64>
}

#[derive(Debug)]
enum Error {
  Io(std::io::Error),
  Json(json::DecoderError),
  Http(hyper::error::Error),
  Api(ZenSendError),
  UnexpectedResponse(StatusCode)
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Error {
    Error::Io(err)
  }
}

impl From<json::DecoderError> for Error {
  fn from(err: json::DecoderError) -> Error {
    Error::Json(err)
  }
}

impl From<hyper::error::Error> for Error {
  fn from(err: hyper::error::Error) -> Error {
    Error::Http(err)
  }
}

impl From<ZenSendError> for Error {
  fn from(err: ZenSendError) -> Error {
    Error::Api(err)
  }
}

 
#[derive(RustcDecodable, RustcEncodable)]
struct APIResult<R> {
  success: Option<R>,
  failure: Option<ZenSendError>
}
 
enum OriginatorType {
  Alpha,
  Msisdn
}

impl Default for OriginatorType {
  fn default() -> OriginatorType { OriginatorType::Alpha }
}

enum SmsEncoding {
  Auto,
  Gsm,
  Ucs2
}

impl Default for SmsEncoding {
  fn default() -> SmsEncoding { SmsEncoding::Auto }
}

#[derive(Default)]
struct Message<'a> {
  originator: &'a str,
  body: &'a str,
  numbers: &'a[&'a str],
  originator_type: OriginatorType,
  sms_encoding: SmsEncoding,
  time_to_live_in_minutes: Option<i32>
}

struct Client {
  client: hyper::Client,
  api_key: String,
  url: String
}

impl Client {
  fn new(api_key: String) -> Client {
    Client { client: hyper::Client::new(), api_key: api_key, url: "https://api.zensend.io".into() }
  }

  fn lookup_operator(&self, number: &str) -> Result<OperatorLookupResult, Error> {
    let vec = vec![("NUMBER", number.to_string())];
    let body = form_urlencoded::serialize(vec.iter());

    let url = self.url.clone() + "/v3/operator_lookup?" + &body;
    let mut res = try!(self.client.get(&url)
      .headers(self.api_headers())
      .send());
  
    self.handle_result(&mut res)
 
  }

  fn check_balance(&self) -> Result<f64, Error> {
    let url = self.url.clone() + "/v3/checkbalance";
    let mut res = try!(self.client.get(&url)
      .headers(self.api_headers())
      .send());

    let result:Balance = try!(self.handle_result(&mut res));
    Ok(result.balance)
  }

  fn get_prices(&self) -> Result<HashMap<String, f64>, Error> {
    let headers = self.api_headers();
    let url = self.url.clone() + "/v3/prices";
    let mut res = try!(self.client.get(&url)
      .headers(headers)
      .send());

    let result:Prices = try!(self.handle_result(&mut res));
    Ok(result.prices_in_pence)
  }

  fn send_sms(&self, message: Message) -> Result<SmsResult, Error> {
 

    let numbers = message.numbers.join(",");
 
    let ttl: String;  
    let mut vec = vec![("BODY", message.body), ("ORIGINATOR", message.originator), ("NUMBERS", numbers.as_ref())];
    
    match message.time_to_live_in_minutes {
      Some(minutes) => {
        ttl = minutes.to_string();
        vec.push(("TIMETOLIVE", ttl.as_ref()))
      },
      None => (),
    }
    
    match message.sms_encoding {
      SmsEncoding::Auto => (),
      SmsEncoding::Gsm => vec.push(("ENCODING", "GSM")),
      SmsEncoding::Ucs2 => vec.push(("ENCODING", "UCS2")),
    }
    
    let body = form_urlencoded::serialize(vec.iter());
     
    let url = self.url.clone() + "/v3/sendsms";
    let mut res = try!(self.client.post(&url)
      .body(body.as_bytes())
      .headers(self.api_headers())
      .send());

    self.handle_result(&mut res)

  }

  fn api_headers(&self) -> Headers {
    let mut headers = Headers::new();
    headers.set_raw("x-api-key", vec![self.api_key.as_bytes().to_vec()]);
    headers
  }

  fn handle_result<T:rustc_serialize::Decodable>(&self, res: &mut Response) -> Result<T, Error> {

    match res.headers.get() {
      Some(&ContentType(Mime(TopLevel::Application, SubLevel::Json, _))) => {
        let mut buffer = String::new();
        try!(res.read_to_string(&mut buffer));
        match try!(json::decode(&buffer)) {
          APIResult{success : Some(sms_result), failure: _} => Ok(sms_result),
          APIResult{failure : Some(error), success: _} => Err(From::from(error)),
          _ => Err(Error::UnexpectedResponse(res.status))
        }

      },
      _ => Err(Error::UnexpectedResponse(res.status))
    }

  }
}

fn main() {
  let client = Client::new("rH_jSYf0d2sbzFw4cSurKQ".into());
  //let res = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796354848"], ..Default::default()});
  //let res = client.check_balance();
  let res = client.get_prices();
  print!("{:?}", res);
}
