#![allow(non_snake_case)]

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
use rustc_serialize::json::Object;

pub enum LocationStrategy {
    Css,
    LinkText,
    PartialLinkText,
    XPath,
}

impl Encodable for LocationStrategy {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match self {
            &LocationStrategy::Css => s.emit_str("css selector"),
            &LocationStrategy::LinkText => s.emit_str("link text"),
            &LocationStrategy::PartialLinkText => s.emit_str("partial link text"),
            &LocationStrategy::XPath => s.emit_str("xpath"),
        }
    }
}

#[derive(RustcDecodable, Debug)]
pub struct WebDriverError {
    pub error: String,
    pub message: String,
}

#[derive(RustcEncodable)]
pub struct NewSessionCmd {
    required: Object,
}

impl NewSessionCmd {
    pub fn new() -> Self {
        NewSessionCmd {
            required: Object::new(),
        }
    }

// TODO firefox specifc prefs
// [moz:firefoxOptions][prefs][name] = value;


}

#[derive(RustcDecodable)]
pub struct Session {
    pub sessionId: String,
}

#[derive(RustcEncodable)]
pub struct GoCmd {
    pub url: String,
}

#[derive(RustcDecodable)]
pub struct Value<T> {
    pub value: T,
}

#[derive(RustcDecodable)]
pub struct CurrentTitle {
    pub title: String,
}

#[derive(RustcEncodable)]
pub struct SwitchWindowCmd<'a> {
    pub handle: &'a str,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Empty {} 

#[derive(RustcEncodable)]
pub struct FindElementCmd<'a> {
    pub using: LocationStrategy,
    pub value: &'a str,
}

pub struct ElementReference {
    pub reference: String,
}

impl Decodable for ElementReference {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("ElementReference", 2, |d| {
            let reference = try!(d.read_struct_field("element-6066-11e4-a52e-4f735466cecf", 0, |d| d.read_str()));
            Ok(ElementReference { reference: reference })
        })
    }
}

#[derive(RustcDecodable)]
pub struct Cookie {
    name: String,
    value: String,
    path: String,
    domain: String,
    secure: bool,
    httpOnly: bool,
    // TODO: expiry:
}
