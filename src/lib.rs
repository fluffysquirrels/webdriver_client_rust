#![deny(warnings)]

//! A library to drive web browsers using the webdriver
//! interface.

// extern crates
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_builder;
extern crate rand;

// Sub-modules
pub mod chrome;
pub mod firefox;
pub mod messages;
pub mod util;

// pub use statements
pub use messages::LocationStrategy;
pub use serde_json::Value as JsonValue;

// use statements
use hyper::client::*;
use hyper::Url;
use messages::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt::{self, Debug};
use std::io::Read;
use std::io;

// --------

/// Error conditions returned by this crate.
#[derive(Debug)]
pub enum Error {
    FailedToLaunchDriver,
    InvalidUrl,
    ConnectionError,
    Io(io::Error),
    JsonDecodeError(serde_json::Error),
    WebDriverError(WebDriverError),
    Base64DecodeError(base64::DecodeError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FailedToLaunchDriver => write!(f, "Unable to start browser driver"),
            Error::InvalidUrl => write!(f, "Invalid URL"),
            Error::ConnectionError => write!(f, "Error connecting to browser"),
            Error::Io(ref err) => write!(f, "{}", err),
            Error::JsonDecodeError(ref s) => write!(f, "Received invalid response from browser: {}", s),
            Error::WebDriverError(ref err) => write!(f, "Error: {}", err.message),
            Error::Base64DecodeError(ref err) => write!(f, "Base64DecodeError: {}", err),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(_: hyper::Error) -> Error {
        Error::ConnectionError
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::JsonDecodeError(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Error {
        Error::Base64DecodeError(e)
    }
}

/// WebDriver server that can create a session.
pub trait Driver {
    /// The url used to connect to this driver
    fn url(&self) -> &str;

    /// Start a session for this driver
    fn session(self, params: &NewSessionCmd) -> Result<DriverSession, Error> where Self : Sized + 'static {
        DriverSession::create_session(Box::new(self), params)
    }
}

/// A driver using a pre-existing WebDriver HTTP URL.
#[derive(Builder)]
#[builder(field(private))]
pub struct HttpDriver {
    #[builder(setter(into))]
    url: String,
}

impl Driver for HttpDriver {
    fn url(&self) -> &str { &self.url }
}

/// Wrapper around the hyper Client, that handles Json encoding and URL construction
struct HttpClient {
    baseurl: Url,
    http: Client,
}

impl HttpClient {
    pub fn new(baseurl: Url) -> Self {
        HttpClient {
            baseurl: baseurl,
            http: Client::new(),
        }
    }

    pub fn decode<D: DeserializeOwned + Debug>(res: &mut Response) -> Result<D, Error> {
        let mut data = String::new();
        res.read_to_string(&mut data)?;
        debug!("result status: {}\n\
                body: '{}'", res.status, data);

        if !res.status.is_success() {
            let err: Value<WebDriverError> = serde_json::from_str(&data)?;
            trace!("deserialize error result: {:#?}", err);
            return Err(Error::WebDriverError(err.value));
        }
        let response = serde_json::from_str(&data);
        trace!("deserialize result: {:#?}", response);
        Ok(response?)
    }

    pub fn get<D: DeserializeOwned + Debug>(&self, path: &str) -> Result<D, Error> {
        let url = self.baseurl.join(path)
                      .map_err(|_| Error::InvalidUrl)?;
        debug!("GET {}", url);
        let mut res = self.http.get(url)
                          .send()?;
        Self::decode(&mut res)
    }

    pub fn delete<D: DeserializeOwned + Debug>(&self, path: &str) -> Result<D, Error> {
        let url = self.baseurl.join(path)
                      .map_err(|_| Error::InvalidUrl)?;
        debug!("DELETE {}", url);
        let mut res = self.http.delete(url)
                          .send()?;
        Self::decode(&mut res)
    }

    pub fn post<D: DeserializeOwned + Debug, E: Serialize>(&self, path: &str, body: &E) -> Result<D, Error> {
        let url = self.baseurl.join(path)
                      .map_err(|_| Error::InvalidUrl)?;
        let body_str = serde_json::to_string(body)?;
        debug!("POST url: {}\n\
                body: {}", url, body_str);
        let mut res = self.http.post(url)
                          .body(&body_str)
                          .send()?;
        Self::decode(&mut res)
    }
}

/// A WebDriver session.
///
/// By default the session is removed on `Drop`
pub struct DriverSession {
    /// driver is kept so it is dropped when DriverSession is dropped.
    _driver: Box<dyn Driver>,
    client: HttpClient,
    session_id: String,
    drop_session: bool,
    capabilities: BTreeMap<String, JsonValue>,
}

impl DriverSession {
    /// Create a new session with the driver.
    pub fn create_session(driver: Box<dyn Driver>, params: &NewSessionCmd)
    -> Result<DriverSession, Error>
    {
        let baseurl = Url::parse(driver.url())
                          .map_err(|_| Error::InvalidUrl)?;
        let client = HttpClient::new(baseurl);
        info!("Creating session at {}", client.baseurl);
        let sess = Self::new_session(&client, params)?;
        info!("Session {} created", sess.sessionId);
        Ok(DriverSession {
            _driver: driver,
            client: client,
            session_id: sess.sessionId,
            drop_session: true,
            capabilities: sess.capabilities,
        })
    }

    /// Use an existing session
    pub fn attach(url: &str, session_id: &str) -> Result<DriverSession, Error> {
        let driver = Box::new(HttpDriver {
            url: url.to_owned(),
        });
        let baseurl = Url::parse(url).map_err(|_| Error::InvalidUrl)?;
        let mut s = DriverSession {
            _driver: driver,
            client: HttpClient::new(baseurl),
            session_id: session_id.to_owned(),
            // This starts as false to avoid triggering the deletion call in Drop
            // if an error occurs
            drop_session: false,
            capabilities: Default::default(),
        };
        info!("Connecting to session at {} with id {}", url, session_id);

        // FIXME /status would be preferable here to test the connection, but
        // it does not seem to work for the current geckodriver

        // We can fetch any value for the session to verify it exists.
        // The page URL will work.
        let _ = s.get_current_url()?;

        info!("Connected to existing session {}", s.session_id);
        // The session exists, enable session deletion on Drop
        s.drop_session = true;
        Ok(s)
    }

    pub fn browser_name(&self) -> Option<&str> {
        if let Some(&JsonValue::String(ref val)) = self.capabilities.get("browserName") {
            Some(val)
        } else {
            None
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Whether to remove the session on Drop, the default is true
    pub fn drop_session(&mut self, drop: bool) {
        self.drop_session = drop;
    }

    /// Create a new webdriver session
    fn new_session(client: &HttpClient, params: &NewSessionCmd) -> Result<Session, Error> {
        let resp: Value<Session> = client.post("/session", &params)?;
        Ok(resp.value)
    }

    /// Navigate to the given URL
    pub fn go(&self, url: &str) -> Result<(), Error> {
        let params = GoCmd { url: url.to_string() };
        let _: Empty = self.client.post(&format!("/session/{}/url", &self.session_id), &params)?;
        Ok(())
    }

    pub fn get_current_url(&self) -> Result<String, Error> {
        let v: Value<_> = self.client.get(&format!("/session/{}/url", self.session_id))?;
        Ok(v.value)
    }

    pub fn back(&self) -> Result<(), Error> {
        let _: Empty = self.client.post(&format!("/session/{}/back", self.session_id), &Empty {})?;
        Ok(())
    }

    pub fn forward(&self) -> Result<(), Error> {
        let _: Empty = self.client.post(&format!("/session/{}/forward", self.session_id), &Empty {})?;
        Ok(())
    }

    pub fn refresh(&self) -> Result<(), Error> {
        let _: Empty = self.client.post(&format!("/session/{}/refresh", self.session_id), &Empty {})?;
        Ok(())
    }

    pub fn get_page_source(&self) -> Result<String, Error> {
        let v: Value<_> = self.client.get(&format!("/session/{}/source", self.session_id))?;
        Ok(v.value)
    }

    pub fn get_title(&self) -> Result<String, Error> {
        let v: Value<_> = self.client.get(&format!("/session/{}/title", self.session_id))?;
        Ok(v.value)
    }

    /// Get all cookies
    pub fn get_cookies(&self) -> Result<Vec<Cookie>, Error> {
        let v: Value<_> = self.client.get(&format!("/session/{}/cookie", self.session_id))?;
        Ok(v.value)
    }

    pub fn get_window_handle(&self) -> Result<String, Error> {
        let v: Value<_> = self.client.get(&format!("/session/{}/window", self.session_id))?;
        Ok(v.value)
    }

    pub fn switch_window(&mut self, handle: &str) -> Result<(), Error> {
        let _: Empty = self.client.post(&format!("/session/{}/window", self.session_id), &SwitchWindowCmd::from(handle))?;
        Ok(())
    }

    pub fn close_window(&mut self) -> Result<(), Error> {
        let _: Empty = self.client.delete(&format!("/session/{}/window", self.session_id))?;
        Ok(())
    }

    pub fn get_window_handles(&self) -> Result<Vec<String>, Error> {
        let v: Value<_> = self.client.get(&format!("/session/{}/window/handles", self.session_id))?;
        Ok(v.value)
    }

    pub fn find_element(&self, selector: &str, strategy: LocationStrategy) -> Result<Element, Error> {
        let cmd = FindElementCmd { using: strategy, value: selector};
        let v: Value<ElementReference> = self.client.post(&format!("/session/{}/element", self.session_id), &cmd)?;
        Ok(Element::new(self, v.value.reference))
    }

    pub fn find_elements(&self, selector: &str, strategy: LocationStrategy) -> Result<Vec<Element>, Error> {
        let cmd = FindElementCmd { using: strategy, value: selector};
        let v: Value<Vec<ElementReference>> = self.client.post(&format!("/session/{}/elements", self.session_id), &cmd)?;

        Ok(v.value.into_iter().map(|er| Element::new(self, er.reference)).collect())
    }

    pub fn execute(&self, script: ExecuteCmd) -> Result<JsonValue, Error> {
        let v: Value<JsonValue> = self.client.post(&format!("/session/{}/execute/sync", self.session_id), &script)?;
        Ok(v.value)
    }

    pub fn execute_async(&self, script: ExecuteCmd) -> Result<JsonValue, Error> {
        let v: Value<JsonValue> = self.client.post(&format!("/session/{}/execute/async", self.session_id), &script)?;
        Ok(v.value)
    }

    /// Valid values are element references as returned by Element::reference() or null to switch
    /// to the top level frame
    pub fn switch_to_frame(&self, handle: JsonValue) -> Result<(), Error> {
        let _: Empty = self.client.post(&format!("/session/{}/frame", self.session_id), &SwitchFrameCmd::from(handle))?;
        Ok(())
    }

    pub fn switch_to_parent_frame(&self) -> Result<(), Error> {
        let _: Empty = self.client.post(&format!("/session/{}/frame/parent", self.session_id), &Empty {})?;
        Ok(())
    }

    /// Take a screenshot of the current frame.
    ///
    /// WebDriver specification: https://www.w3.org/TR/webdriver/#take-screenshot
    pub fn screenshot(&self) -> Result<Screenshot, Error> {
        let v: Value<String> = self.client.get(&format!("/session/{}/screenshot",
                                                        self.session_id))?;
        Screenshot::from_string(v.value)
    }
}

impl Drop for DriverSession {
    fn drop(&mut self) {
        if self.drop_session {
            let _: Result<Empty,_> = self.client.delete(&format!("/session/{}", self.session_id));
        }
    }
}

/// An HTML element within a WebDriver session.
pub struct Element<'a> {
    session: &'a DriverSession,
    reference: String,
}

impl<'a> Element<'a> {
    pub fn new(s: &'a DriverSession, reference: String) -> Self {
        Element { session: s, reference: reference }
    }

    pub fn attribute(&self, name: &str) -> Result<String, Error> {
        let v: Value<_> = self.session.client.get(&format!("/session/{}/element/{}/attribute/{}", self.session.session_id(), self.reference, name))?;
        Ok(v.value)
    }

    /// Return this element's property value.
    ///
    /// WebDriver spec: https://www.w3.org/TR/webdriver/#get-element-property
    pub fn property(&self, name: &str) -> Result<String, Error> {
        let v: Value<_> = self.session.client.get(&format!("/session/{}/element/{}/property/{}", self.session.session_id(), self.reference, name))?;
        Ok(v.value)
    }

    /// Click this element.
    ///
    /// WebDriver spec: https://www.w3.org/TR/webdriver/#element-click
    pub fn click(&self) -> Result<(), Error> {
        let _: Value<JsonValue> = self.session.client.post(
            &format!("/session/{}/element/{}/click", self.session.session_id(), self.reference),
            &Empty {})?;
        Ok(())
    }

    /// Clear the text of this element.
    ///
    /// WebDriver spec: https://www.w3.org/TR/webdriver/#element-clear
    pub fn clear(&self) -> Result<(), Error> {
        let _: Value<JsonValue> = self.session.client.post(&format!("/session/{}/element/{}/clear", self.session.session_id(), self.reference), &Empty {})?;
        Ok(())
    }

    /// Send key presses to this element.
    ///
    /// WebDriver spec: https://www.w3.org/TR/webdriver/#element-send-keys
    pub fn send_keys(&self, s: &str) -> Result<(), Error> {
        let _: Value<JsonValue> =
            self.session.client.post(&format!("/session/{}/element/{}/value",
                                              self.session.session_id(), self.reference),
                                     &json!({ "text": s }))?;
        Ok(())
    }

    pub fn css_value(&self, name: &str) -> Result<String, Error> {
        let v: Value<_> = self.session.client.get(&format!("/session/{}/element/{}/css/{}", self.session.session_id(), self.reference, name))?;
        Ok(v.value)
    }

    pub fn text(&self) -> Result<String, Error> {
        let v: Value<_> = self.session.client.get(&format!("/session/{}/element/{}/text", self.session.session_id(), self.reference))?;
        Ok(v.value)
    }

    /// Returns the tag name for this element
    pub fn name(&self) -> Result<String, Error> {
        let v: Value<_> = self.session.client.get(&format!("/session/{}/element/{}/name", self.session.session_id(), self.reference))?;
        Ok(v.value)
    }

    pub fn find_element(&self, selector: &str, strategy: LocationStrategy) -> Result<Element, Error> {
        let cmd = FindElementCmd { using: strategy, value: selector };
        let v: Value<ElementReference> = self.session.client.post(&format!("/session/{}/element/{}/element", self.session.session_id, self.reference), &cmd)?;
        Ok(Element::new(self.session, v.value.reference))
    }

    pub fn find_elements(&self, selector: &str, strategy: LocationStrategy) -> Result<Vec<Element>, Error> {
        let cmd = FindElementCmd { using: strategy, value: selector };
        let v: Value<Vec<ElementReference>> = self.session.client.post(&format!("/session/{}/element/{}/elements", self.session.session_id, self.reference), &cmd)?;

        Ok(v.value.into_iter().map(|er| Element::new(self.session, er.reference)).collect())
    }

    /// Returns a reference that can be passed on to the API
    pub fn reference(&self) -> Result<JsonValue, Error> {
        serde_json::to_value(&ElementReference::from_str(&self.reference))
            .map_err(|err| Error::from(err))
    }

    /// The raw reference id that identifies this element, this can be used
    /// with Element::new()
    pub fn raw_reference(&self) -> &str { &self.reference }

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

    /// Take a screenshot of this element
    ///
    /// WebDriver specification: https://www.w3.org/TR/webdriver/#take-element-screenshot
    pub fn screenshot(&self) -> Result<Screenshot, Error> {
        let v: Value<String> = self.session.client.get(
            &format!("/session/{}/element/{}/screenshot",
                     self.session.session_id,
                     self.reference))?;
        Screenshot::from_string(v.value)
    }
}

impl<'a> fmt::Debug for Element<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WebDriver Element with remote reference {}", self.reference)
    }
}

/// Switch the context of the current session to the given frame reference.
///
/// This structure implements Drop, and restores the session context
/// to the current top level window.
pub struct FrameContext<'a> {
    session: &'a DriverSession,
}

impl<'a> FrameContext<'a> {
    pub fn new(session: &'a DriverSession, frameref: JsonValue) -> Result<FrameContext<'a>, Error> {
        session.switch_to_frame(frameref)?;
        Ok(FrameContext { session: session })
    }
}

impl<'a> Drop for FrameContext<'a> {
    fn drop(&mut self) {
        let _ = self.session.switch_to_frame(JsonValue::Null);
    }
}

pub struct Screenshot {
    base64: String,
}

impl Screenshot {
    fn from_string(s: String) -> Result<Screenshot, Error> {
        Ok(Screenshot {
            base64: s,
        })
    }

    pub fn bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(base64::decode(&self.base64)?)
    }

    pub fn save_file(&self, path: &str) -> Result<(), Error> {
        Ok(std::fs::write(path, self.bytes()?)?)
    }
}
