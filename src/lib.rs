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
pub struct SmsResult {
  pub txguid: String,
  pub numbers: i32,
  pub smsparts: i32,
  pub encoding: String,
  pub cost_in_pence: f64,
  pub new_balance_in_pence: f64
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct CreateKeywordResult {
  pub cost_in_pence: f64,
  pub new_balance_in_pence: f64
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
pub struct OperatorLookupResult {
  pub mcc: String,
  pub mnc: String,
  pub operator: String,
  pub cost_in_pence: f64,
  pub new_balance_in_pence: f64
}

#[derive(RustcDecodable, RustcEncodable, Debug, PartialEq)]
pub struct ZenSendError {
  pub failcode: String,
  pub parameter: Option<String>,
  pub cost_in_pence: Option<f64>,
  pub new_balance_in_pence: Option<f64>
}


#[derive(Debug)]
pub enum Error {
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
 
#[derive(PartialEq)]
pub enum OriginatorType {
  Alpha,
  Msisdn
}

impl Default for OriginatorType {
  fn default() -> OriginatorType { OriginatorType::Alpha }
}

pub enum SmsEncoding {
  Auto,
  Gsm,
  Ucs2
}

impl Default for SmsEncoding {
  fn default() -> SmsEncoding { SmsEncoding::Auto }
}

#[derive(Default)]
pub struct Message<'a> {
  pub originator: &'a str,
  pub body: &'a str,
  pub numbers: &'a[&'a str],
  pub originator_type: OriginatorType,
  pub sms_encoding: SmsEncoding,
  pub time_to_live_in_minutes: Option<i32>
}

#[derive(Default)]
pub struct CreateKeywordRequest<'a> {
  pub shortcode: &'a str,
  pub keyword: &'a str,
  pub is_sticky: bool,
  pub mo_url: Option<&'a str>,
}

pub struct Client {
  client: hyper::Client,
  api_key: String,
  url: String
}

impl Client {
  pub fn new(api_key: String) -> Client {
    Client { client: hyper::Client::new(), api_key: api_key, url: "https://api.zensend.io".into() }
  }

  pub fn new_with_client(api_key: String, client: hyper::Client) -> Client {
    Client { client: client, api_key: api_key, url: "https://api.zensend.io".into() }
  }

  pub fn new_with_url(api_key: String, url: String) -> Client {
    Client { client: hyper::Client::new(), api_key: api_key, url: url }
  }

 
  pub fn new_with_client_and_url(api_key: String, client: hyper::Client, url: String) -> Client {
    Client { client: client, api_key: api_key, url: url }
  }

  pub fn lookup_operator(&self, number: &str) -> Result<OperatorLookupResult, Error> {
    let mut form_serializer = form_urlencoded::Serializer::new(String::new());

    form_serializer.append_pair("NUMBER", &number.to_string());
    let body = form_serializer.finish();

    let url = self.url.clone() + "/v3/operator_lookup?" + &body;
    let mut res = try!(self.client.get(&url)
      .headers(self.api_headers())
      .send());
  
    self.handle_result(&mut res)
 
  }

  pub fn check_balance(&self) -> Result<f64, Error> {
    let url = self.url.clone() + "/v3/checkbalance";
    let mut res = try!(self.client.get(&url)
      .headers(self.api_headers())
      .send());

    let result:Balance = try!(self.handle_result(&mut res));
    Ok(result.balance)
  }

  pub fn get_prices(&self) -> Result<HashMap<String, f64>, Error> {
    let headers = self.api_headers();
    let url = self.url.clone() + "/v3/prices";
    let mut res = try!(self.client.get(&url)
      .headers(headers)
      .send());

    let result:Prices = try!(self.handle_result(&mut res));
    Ok(result.prices_in_pence)
  }

  pub fn create_keyword(&self, keyword_request : CreateKeywordRequest) -> Result<CreateKeywordResult, Error> {
    let sticky = keyword_request.is_sticky.to_string();
    let mut vec = vec![("SHORTCODE", keyword_request.shortcode), ("KEYWORD", keyword_request.
    keyword), ("IS_STICKY", &sticky)];

    match keyword_request.mo_url {
      Some(url) => {
        vec.push(("MO_URL", url))
      },
      None => (),
    }

    let body = form_urlencoded::serialize(vec.iter());
    let url = self.url.clone() + "/v3/keywords";

    let mut res = try!(self.client.post(&url)
      .body(body.as_bytes())
      .headers(self.api_headers())
      .send());

    self.handle_result(&mut res)
  }

  pub fn send_sms(&self, message: Message) -> Result<SmsResult, Error> {
 

    let numbers = message.numbers.join(",");
    let mut form_serializer = form_urlencoded::Serializer::new(String::new());
 

    form_serializer
	.append_pair("BODY", message.body)
	.append_pair("ORIGINATOR", message.originator)
	.append_pair("NUMBERS", numbers.as_ref())
	.append_pair("ORIGINATOR_TYPE", if message.originator_type == OriginatorType::Alpha  {"alpha"} else {"msisdn"});

    
    match message.time_to_live_in_minutes {
      Some(minutes) => {
        let ttl = minutes.to_string();
	form_serializer.append_pair("TIMETOLIVE", ttl.as_ref());
      },
      None => (),
    }


    match message.sms_encoding {
      SmsEncoding::Auto => (),
      SmsEncoding::Gsm => { form_serializer.append_pair("ENCODING", "gsm"); },
      SmsEncoding::Ucs2 =>{ form_serializer.append_pair("ENCODING", "ucs2"); },
    }
    
    let body = form_serializer.finish();
     
    let url = self.url.clone() + "/v3/sendsms";
    let mut res = try!(self.client.post(&url)
      .body(body.as_bytes())
      .headers(self.api_headers())
      .send());

    self.handle_result(&mut res)

  }

  fn api_headers(&self) -> Headers {
    let mut headers = Headers::new();
    headers.set_raw("X-API-KEY", vec![self.api_key.as_bytes().to_vec()]);
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


