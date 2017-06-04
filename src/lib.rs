//! A library to drive web browsers using the webdriver
//! interface.

use std::convert::From;
use std::io::Read;
use std::io;
use std::fmt;

extern crate hyper;
use hyper::client::*;
use hyper::Url;

extern crate serde;
use serde::Serialize;
use serde::de::DeserializeOwned;

extern crate serde_json;
pub use serde_json::Value as JsonValue;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;



extern crate rand;

pub mod messages;
use messages::*;

pub mod firefox;

#[derive(Debug)]
pub enum Error {
    FailedToLaunchDriver,
    InvalidUrl,
    ConnectionError,
    JsonDecodeError(serde_json::Error),
    WebDriverError(WebDriverError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FailedToLaunchDriver => write!(f, "Unable to start browser driver"),
            Error::InvalidUrl => write!(f, "Invalid URL"),
            Error::ConnectionError => write!(f, "Error connecting to browser"),
            Error::JsonDecodeError(ref s) => write!(f, "Received invalid response from browser: {}", s),
            Error::WebDriverError(ref err) => write!(f, "Error: {}", err.message),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(_: hyper::Error) -> Error {
        Error::ConnectionError
    }
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::ConnectionError
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::JsonDecodeError(e)
    }
}

pub trait Driver {
    fn url(&self) -> &str;
}

/// A WebDriver session.
///
/// The session is removed on `Drop`
pub struct DriverSession<T> {
    driver: Option<T>,
    baseurl: Url,
    client: Client,
    session_id: String,
}

impl<T> DriverSession<T> where T: Driver {
    pub fn new(driver: T) -> Result<Self,Error> {
        let mut s = try!(Self::for_url(driver.url()));
        s.driver = Some(driver);
        Ok(s)
    }
}

impl<T> DriverSession<T> {
    /// Connect to an existing WebDriver
    pub fn for_url(url: &str) -> Result<Self, Error> {
        let baseurl = try!(Url::parse(url).map_err(|_| Error::InvalidUrl));
        let mut s = DriverSession {
            driver: None,
            baseurl: baseurl,
            client: Client::new(),
            session_id: String::new(),
        };
        info!("Creating session at {}", url);
        let sess = try!(s.new_session(&NewSessionCmd::new()));
        s.session_id = sess.sessionId;
        info!("Session {} created", s.session_id);
        Ok(s)
    }

    /// Connect to an existing WebDriver session
    pub fn for_session(url: &str, session_id: &str) -> Result<Self, Error> {
        let baseurl = try!(Url::parse(url).map_err(|_| Error::InvalidUrl));
        let s = DriverSession {
            driver: None,
            baseurl: baseurl,
            client: Client::new(),
            session_id: session_id.to_owned(),
        };
        info!("Connecting to session at {} with id {}", url, session_id);
        // We can fetch any value for the session to verify it exists.
        // The page URL will work.
        s.get_current_url()?;
        info!("Connected to existing session {}", s.session_id);
        Ok(s)
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    fn get<D: DeserializeOwned>(&self, path: &str) -> Result<D, Error> {
        let url = try!(self.baseurl.join(path)
                           .map_err(|_| Error::InvalidUrl));
        debug!("GET {}", url);
        let mut res = try!(self.client.get(url)
                            .send());
        Self::decode(&mut res)
    }

    fn delete<D: DeserializeOwned>(&self, path: &str) -> Result<D, Error> {
        let url = try!(self.baseurl.join(path)
                           .map_err(|_| Error::InvalidUrl));
        debug!("DELETE {}", url);
        let mut res = try!(self.client.delete(url)
                            .send());
        Self::decode(&mut res)
    }

    fn decode<D: DeserializeOwned>(res: &mut Response) -> Result<D, Error> {
        let mut data = String::new();
        try!(res.read_to_string(&mut data));
        debug!("result body: '{}'", data);
        if !res.status.is_success() {
            let err = try!(serde_json::from_str(&data));
            return Err(Error::WebDriverError(err));
        }
        let response = try!(serde_json::from_str(&data));
        Ok(response)
    }

    fn post<D: DeserializeOwned, E: Serialize>(&self, path: &str, body: &E) -> Result<D, Error> {
        let url = try!(self.baseurl.join(path)
                           .map_err(|_| Error::InvalidUrl));
        let body_str = try!(serde_json::to_string(body));
        debug!("POST url: {}\n\
                body: {}", url, body_str);
        let mut res = try!(self.client.post(url)
                            .body(&body_str)
                            .send());
        Self::decode(&mut res)
    }

    /// Create a new webdriver session
    fn new_session(&mut self, params: &NewSessionCmd) -> Result<Session, Error> {
        let resp: Session = try!(self.post("/session", &params));
        Ok(resp)
    }

    /// Navigate to the given URL
    pub fn go(&self, url: &str) -> Result<(), Error> {
        let params = GoCmd { url: url.to_string() };
        let _: Empty = try!(self.post(&format!("/session/{}/url", &self.session_id), &params));
        Ok(())
    }

    pub fn get_current_url(&self) -> Result<String, Error> {
        let v: Value<_> = try!(self.get(&format!("/session/{}/url", self.session_id)));
        Ok(v.value)
    }

    pub fn back(&self) -> Result<(), Error> {
        let _: Empty = try!(self.post(&format!("/session/{}/back", self.session_id), &Empty {}));
        Ok(())
    }

    pub fn forward(&self) -> Result<(), Error> {
        let _: Empty = try!(self.post(&format!("/session/{}/forward", self.session_id), &Empty {}));
        Ok(())
    }

    pub fn refresh(&self) -> Result<(), Error> {
        let _: Empty = try!(self.post(&format!("/session/{}/refresh", self.session_id), &Empty {}));
        Ok(())
    }

    pub fn get_page_source(&self) -> Result<String, Error> {
        let v: Value<_> = try!(self.get(&format!("/session/{}/source", self.session_id)));
        Ok(v.value)
    }

    pub fn get_title(&self) -> Result<String, Error> {
        let v: Value<_> = try!(self.get(&format!("/session/{}/title", self.session_id)));
        Ok(v.value)
    }

    /// Get all cookies
    pub fn get_cookies(&self) -> Result<Vec<Cookie>, Error> {
        let v: Value<_> = try!(self.get(&format!("/session/{}/cookie", self.session_id)));
        Ok(v.value)
    }

    pub fn get_window_handle(&self) -> Result<String, Error> {
        let v: Value<_> = try!(self.get(&format!("/session/{}/window", self.session_id)));
        Ok(v.value)
    }

    pub fn switch_window(&mut self, handle: &str) -> Result<(), Error> {
        let _: Empty = try!(self.post(&format!("/session/{}/window", self.session_id), &SwitchWindowCmd::from(handle)));
        Ok(())
    }

    pub fn close_window(&mut self) -> Result<(), Error> {
        let _: Empty = try!(self.delete(&format!("/session/{}/window", self.session_id)));
        Ok(())
    }

    pub fn get_window_handles(&self) -> Result<Vec<String>, Error> {
        let v: Value<_> = try!(self.get(&format!("/session/{}/window/handles", self.session_id)));
        Ok(v.value)
    }

    pub fn find_element(&self, selector: &str, strategy: LocationStrategy) -> Result<Element<T>, Error> {
        let cmd = FindElementCmd { using: strategy, value: selector};
        let v: Value<ElementReference> = try!(self.post(&format!("/session/{}/element", self.session_id), &cmd));
        Ok(Element::new(self, v.value.reference))
    }

    pub fn find_elements(&self, selector: &str, strategy: LocationStrategy) -> Result<Vec<Element<T>>, Error> {
        let cmd = FindElementCmd { using: strategy, value: selector};
        let mut v: Value<Vec<ElementReference>> = try!(self.post(&format!("/session/{}/elements", self.session_id), &cmd));

        let mut elems = Vec::new();
        while let Some(er) = v.value.pop() {
            elems.push(Element::new(self, er.reference))
        }
        Ok(elems)
    }

    pub fn execute(&self, script: ExecuteCmd) -> Result<JsonValue, Error> {
        let v: Value<JsonValue> = try!(self.post(&format!("/session/{}/execute/sync", self.session_id), &script));
        Ok(v.value)
    }

    pub fn switch_to_frame(&self, handle: JsonValue) -> Result<(), Error> {
        let _: Empty = try!(self.post(&format!("/session/{}/frame", self.session_id), &SwitchFrameCmd::from(handle)));
        Ok(())
    }
}

impl<T> Drop for DriverSession<T> {
    fn drop(&mut self) {
        let _: Result<Empty,_> = self.delete(&format!("/session/{}", self.session_id));
    }
}

pub struct Element<'a, T: 'a> {
    session: &'a DriverSession<T>,
    reference: String,
}

impl<'a, T> Element<'a, T> {
    fn new(s: &'a DriverSession<T>, reference: String) -> Self {
        Element { session: s, reference: reference }
    }

    pub fn attribute(&self, name: &str) -> Result<String, Error> {
        let v: Value<_> = try!(self.session.get(&format!("/session/{}/element/{}/attribute/{}", self.session.session_id(), self.reference, name)));
        Ok(v.value)
    }

//    pub fn property(&self, name: &str) -> Result<String, Error> {
//        let v: Value<_> = try!(self.get(&format!("/session/{}/element/{}/property/{}", self.session_id, el.reference, name)));
//        Ok(v.value)
//    }

    pub fn css_value(&self, name: &str) -> Result<String, Error> {
        let v: Value<_> = try!(self.session.get(&format!("/session/{}/element/{}/css/{}", self.session.session_id(), self.reference, name)));
        Ok(v.value)
    }

    pub fn text(&self) -> Result<String, Error> {
        let v: Value<_> = try!(self.session.get(&format!("/session/{}/element/{}/text", self.session.session_id(), self.reference)));
        Ok(v.value)
    }

    /// Returns the tag name for this element
    pub fn name(&self) -> Result<String, Error> {
        let v: Value<_> = try!(self.session.get(&format!("/session/{}/element/{}/name", self.session.session_id(), self.reference)));
        Ok(v.value)
    }

    pub fn reference(&self) -> Result<JsonValue, Error> {
        serde_json::to_value(&ElementReference::from_str(&self.reference))
            .map_err(|err| Error::from(err))
    }

    /// Gets the `innerHTML` javascript attribute for this element. Some drivers can get
    /// this using regular attributes, in others it does not work. This method gets it
    /// executing a bit of javascript.
    pub fn inner_html(&self) -> Result<JsonValue, Error> {
        let script = ExecuteCmd {
            script: "return arguments[0].innerHTML;".to_owned(),
            args: vec![self.reference()?],
        };
        self.session.execute(script)
    }

    pub fn outer_html(&self) -> Result<JsonValue, Error> {
        let script = ExecuteCmd {
            script: "return arguments[0].outerHTML;".to_owned(),
            args: vec![self.reference()?],
        };
        self.session.execute(script)
    }
}

/// Switch the context of the current session to the given frame reference.
///
/// This structure implements Drop, and restores the session context
/// to the current top level window.
pub struct FrameContext<'a, T: 'a> {
    session: &'a DriverSession<T>,
}

impl<'a, T> FrameContext<'a, T> {
    pub fn new(session: &DriverSession<T>, frameref: JsonValue) -> Result<FrameContext<T>, Error> {
        session.switch_to_frame(frameref)?;
        Ok(FrameContext { session: session })
    }
}

impl<'a, T> Drop for FrameContext<'a, T> {
    fn drop(&mut self) {
        let _ = self.session.switch_to_frame(JsonValue::Null);
    }
}
