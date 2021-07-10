extern crate zensend;
extern crate hyper;
#[macro_use]
extern crate matches;

use zensend::Client;
use zensend::Message;
use zensend::OriginatorType;
use zensend::SmsEncoding;
use zensend::Error;
use zensend::ZenSendError;
use zensend::CreateKeywordRequest;

use hyper::server::{Server, Request, Response, Handler, Listening};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::header::{ContentType};
use hyper::status::StatusCode;
use hyper::uri::RequestUri;

use std::io::Read;

use std::sync::{Mutex, Arc};

use std::collections::HashMap;

struct MockHandler<'a> {
  response: &'a [u8],
  saved_request: Arc<Mutex<SavedRequest>>
}

struct SavedRequest {
  body: String,
  uri: RequestUri
}

impl SavedRequest {
  fn new() -> SavedRequest {
    SavedRequest{ body: String::new(), uri: RequestUri::Star}
  }
}
impl <'a> Handler for MockHandler<'a> {
  
  fn handle(&self, mut req: Request, mut res: Response) {
    let mut saved_request = self.saved_request.lock().unwrap();
    req.read_to_string(&mut saved_request.body).unwrap();
    saved_request.uri = req.uri;
    res.headers_mut().set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
    res.send(self.response).unwrap()    
  }
}

impl <'a> MockHandler<'a> {
  fn new(response: &'a [u8], mutex: Arc<Mutex<SavedRequest>>) -> MockHandler<'a> {
    MockHandler {response: response, saved_request: mutex}
 
  }
}

fn create_client(response: &'static [u8]) -> (Listening, Client, Arc<Mutex<SavedRequest>>) {

  let server = Server::http("127.0.0.1:0").unwrap();

  let mutex = Arc::new(Mutex::new(SavedRequest::new()));
  let handler = MockHandler::new(response, mutex.clone());
  let r = server.handle(handler).unwrap();
  let port =  r.socket.port();
  let url = "http://127.0.0.1:".to_string() + &port.to_string();
  let client = Client::new_with_client_and_url("api_key".into(), hyper::Client::new(), url);

  (r, client, mutex)  
}

#[test]
fn it_can_create_a_keyword() {

  let response = br#"{
    "success": {
        "cost_in_pence": 5.4,
        "new_balance_in_pence": 10.2
    }
  }"#;

  let (mut r, client, mutex) = create_client(response);


  let wrapped_result = client.create_keyword(CreateKeywordRequest { shortcode: "SC", keyword: "KW", ..Default::default()});

  r.close().unwrap();
  let result = wrapped_result.unwrap();
  let data = mutex.lock().unwrap();

  assert_eq!(5.4, result.cost_in_pence);
  assert_eq!(10.2, result.new_balance_in_pence);

  assert_eq!("SHORTCODE=SC&KEYWORD=KW&IS_STICKY=false", (*data).body);
}

#[test]
fn it_can_create_a_keyword_with_mo_url() {

  let response = br#"{
    "success": {
        "cost_in_pence": 5.4,
        "new_balance_in_pence": 10.2
    }
  }"#;

  let (mut r, client, mutex) = create_client(response);


  let wrapped_result = client.create_keyword(CreateKeywordRequest { shortcode: "SC", keyword: "KW", is_sticky: true, mo_url: Some("http://mo")});

  r.close().unwrap();
  let result = wrapped_result.unwrap();
  let data = mutex.lock().unwrap();

  assert_eq!(5.4, result.cost_in_pence);
  assert_eq!(10.2, result.new_balance_in_pence);

  assert_eq!("SHORTCODE=SC&KEYWORD=KW&IS_STICKY=true&MO_URL=http%3A%2F%2Fmo", (*data).body);
}

#[test]
fn it_can_send_an_sms() {

  let response = br#"{
    "success": {
        "txguid": "7CDEB38F-4370-18FD-D7CE-329F21B99209",
        "numbers": 1,
        "smsparts": 1,
        "encoding": "gsm",
        "cost_in_pence": 5.4,
        "new_balance_in_pence": 10.2
    }
  }"#;

  let (mut r, client, mutex) = create_client(response);

  let wrapped_result = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796351234"], ..Default::default()});

  r.close().unwrap();
  let result = wrapped_result.unwrap();
  let data = mutex.lock().unwrap();

  assert_eq!(1, result.numbers);
  assert_eq!(1, result.smsparts);
  assert_eq!("gsm", result.encoding);
  assert_eq!(5.4, result.cost_in_pence);
  assert_eq!(10.2, result.new_balance_in_pence);
  assert_eq!("7CDEB38F-4370-18FD-D7CE-329F21B99209", result.txguid);

  assert_eq!("BODY=Hello&ORIGINATOR=ZenSend&NUMBERS=447796351234&ORIGINATOR_TYPE=alpha", (*data).body);
}


#[test]
fn it_can_send_an_sms_with_optional_params() {

  let response = br#"{
    "success": {
        "txguid": "7CDEB38F-4370-18FD-D7CE-329F21B99209",
        "numbers": 1,
        "smsparts": 1,
        "encoding": "gsm",
        "cost_in_pence": 5.4,
        "new_balance_in_pence": 10.2
    }
  }"#;

  let (mut r, client, mutex) = create_client(response);

  let wrapped_result = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796351234"], originator_type: OriginatorType::Alpha, sms_encoding: SmsEncoding::Gsm, time_to_live_in_minutes: Some(60)});

  r.close().unwrap();
  let result = wrapped_result.unwrap();
  let data = mutex.lock().unwrap();

  assert_eq!(1, result.numbers);
  assert_eq!(1, result.smsparts);
  assert_eq!("gsm", result.encoding);
  assert_eq!(5.4, result.cost_in_pence);
  assert_eq!(10.2, result.new_balance_in_pence);
  assert_eq!("7CDEB38F-4370-18FD-D7CE-329F21B99209", result.txguid);

  assert_eq!("BODY=Hello&ORIGINATOR=ZenSend&NUMBERS=447796351234&ORIGINATOR_TYPE=alpha&TIMETOLIVE=60&ENCODING=gsm", (*data).body);
}

#[test]
fn it_can_send_multiple_sms() {

  let response = br#"{
    "success": {
        "txguid": "7CDEB38F-4370-18FD-D7CE-329F21B99209",
        "numbers": 2,
        "smsparts": 1,
        "encoding": "gsm",
        "cost_in_pence": 5.4,
        "new_balance_in_pence": 10.2
    }
  }"#;

  let (mut r, client, mutex) = create_client(response);

  let wrapped_result = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796351234", "447796351235"], ..Default::default()});

  r.close().unwrap();
  let result = wrapped_result.unwrap();
  let data = mutex.lock().unwrap();

  assert_eq!(2, result.numbers);
  assert_eq!(1, result.smsparts);
  assert_eq!("gsm", result.encoding);
  assert_eq!(5.4, result.cost_in_pence);
  assert_eq!(10.2, result.new_balance_in_pence);
  assert_eq!("7CDEB38F-4370-18FD-D7CE-329F21B99209", result.txguid);

  assert_eq!("BODY=Hello&ORIGINATOR=ZenSend&NUMBERS=447796351234%2C447796351235&ORIGINATOR_TYPE=alpha", (*data).body);
}

fn as_zensend_error(error: Error) -> ZenSendError {
  match error {
    Error::Api(e) => e,
    _ => panic!("unexpected type")
  }
}

#[test]
fn it_should_be_able_to_handle_an_error() {

  let response = br#"{
    "failure": {
        "failcode": "GENERIC_ERROR"
    }
  }"#;

  let (mut r, client, _mutex) = create_client(response);

  let wrapped_result = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796351234"], ..Default::default()});

  r.close().unwrap();
  let result = wrapped_result.unwrap_err();

  assert_eq!(ZenSendError { failcode: "GENERIC_ERROR".to_string(), parameter: None, cost_in_pence: None, new_balance_in_pence: None }, as_zensend_error(result))

}

#[test]
fn it_should_be_able_to_handle_a_parameter_error() {

  let response = br#"{
    "failure": {
        "failcode": "IS_EMPTY",
        "parameter": "BODY"
    }
  }"#;

  let (mut r, client, _mutex) = create_client(response);

  let wrapped_result = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796351234"], ..Default::default()});

  r.close().unwrap();
  let result = wrapped_result.unwrap_err();

  assert_eq!(ZenSendError { failcode: "IS_EMPTY".to_string(), parameter: Some("BODY".to_string()), cost_in_pence: None, new_balance_in_pence: None }, as_zensend_error(result))

}

#[test]
fn it_should_be_able_to_handle_an_invalid_json_response() {

  let response = br#"{
    "failures": {
        "failcode": "IS_EMPTY",
        "parameter": "BODY"
    }
  }"#;

  let (mut r, client, _mutex) = create_client(response);

  let wrapped_result = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796351234"], ..Default::default()});

  r.close().unwrap();
  let result = wrapped_result.unwrap_err();

  assert!(matches!(result, Error::UnexpectedResponse(StatusCode::Ok)))


}

#[test]
fn it_should_be_able_to_handle_malformed_json_response() {

  let response = br#"NOT JSON"#;

  let (mut r, client, _mutex) = create_client(response);

  let wrapped_result = client.send_sms(Message { originator: "ZenSend", body: "Hello", numbers: &["447796351234"], ..Default::default()});

  r.close().unwrap();
  let result = wrapped_result.unwrap_err();

  assert!(matches!(result, Error::Json(_)))

}

#[test]
fn it_should_be_able_to_retrieve_the_balance() {

  let response = br#"{
      "success": {
          "balance": 100.2
      }
    }"#;

  let (mut r, client, _mutex) = create_client(response);

  let wrapped_result = client.check_balance();

  r.close().unwrap();
  let result = wrapped_result.unwrap();

  assert_eq!(100.2, result);

}


#[test]
fn it_should_be_able_to_retrieve_the_prices() {

  let response = br#"{
      "success": {
          "prices_in_pence": {"GB":1.23,"US":1.24}
      }
    }"#;

  let (mut r, client, _mutex) = create_client(response);

  let wrapped_result = client.get_prices();

  r.close().unwrap();
  let result = wrapped_result.unwrap();

  let mut hm = HashMap::new();
  hm.insert("GB".to_string(), 1.23);
  hm.insert("US".to_string(), 1.24);

  assert_eq!(hm, result);

}

#[test]
fn it_should_be_able_to_do_an_operator_lookup() {

  let response = br#"{
        "success": {
          "mnc":"123",
          "mcc":"456",
          "operator":"o2-uk",
          "cost_in_pence":2.5,
          "new_balance_in_pence":100.0
        }
      }"#;

  let (mut r, client, mutex) = create_client(response);

  let wrapped_result = client.lookup_operator("441234567890");

  r.close().unwrap();
  let result = wrapped_result.unwrap();

  let data = mutex.lock().unwrap();

  assert_eq!("123", result.mnc);
  assert_eq!("456", result.mcc);
  assert_eq!("o2-uk", result.operator);
  assert_eq!(2.5, result.cost_in_pence);
  assert_eq!(100.0, result.new_balance_in_pence);

  assert_eq!(RequestUri::AbsolutePath("/v3/operator_lookup?NUMBER=441234567890".to_string()), (*data).uri)

}

#[test]
fn it_should_be_able_to_handle_an_error_from_an_operator_lookup() {

  let response = br#"{
        "failure": {
          "failcode":"DATA_MISSING",
          "cost_in_pence":2.5,
          "new_balance_in_pence":100.0
        }
      }"#;

  let (mut r, client, _mutex) = create_client(response);

  let wrapped_result = client.lookup_operator("441234567890");

  r.close().unwrap();
  let result = wrapped_result.unwrap_err();


  assert_eq!(ZenSendError { failcode: "DATA_MISSING".to_string(), cost_in_pence: Some(2.5), new_balance_in_pence: Some(100.0), parameter: None }, as_zensend_error(result))




}
