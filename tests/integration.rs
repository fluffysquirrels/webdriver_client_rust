extern crate env_logger;
extern crate hyper;
extern crate log;
#[macro_use]
extern crate serde_json;
extern crate webdriver_client;

use env_logger::{LogBuilder, LogTarget};
use log::LogLevelFilter;
use std::io::Read;
use std::env;
use std::path::PathBuf;
use std::sync::Once;
use std::thread::sleep;
use std::time::Duration;
use webdriver_client::{Driver, DriverSession, HttpDriverBuilder, LocationStrategy};
use webdriver_client::firefox::GeckoDriver;
use webdriver_client::chrome::ChromeDriver;
use webdriver_client::messages::{ExecuteCmd, NewSessionCmd};

/// The different browsers supported in tests
#[derive(Debug)]
enum TestBrowser {
    Firefox,
    Chrome,
}

impl TestBrowser {
    fn session(&self) -> DriverSession {
        match *self {
            TestBrowser::Firefox => {
                GeckoDriver::build()
                            .spawn()
                            .expect("Error starting geckodriver")
                            .session(&self.new_session_cmd())
                            .expect("Error starting session")
            }
            TestBrowser::Chrome => {
                ChromeDriver::build()
                             .spawn()
                             .expect("Error starting chromedriver")
                             .session(&self.new_session_cmd())
                             .expect("Error starting session")
            }
        }
    }

    fn driver(&self) -> Box<dyn Driver> {
        match *self {
            TestBrowser::Firefox => {
                Box::new(GeckoDriver::build()
                         .spawn().expect("Error starting geckodriver"))
            }
            TestBrowser::Chrome => {
                Box::new(ChromeDriver::build()
                         .spawn().expect("Error starting chromedriver"))
            }
        }
    }

    fn new_session_cmd(&self) -> NewSessionCmd {
        let mut new = NewSessionCmd::default();

        match *self {
            TestBrowser::Firefox => {
                new.always_match(
                    "moz:firefoxOptions", json!({
                        "args": ["-headless"]
                    }))
            }
            TestBrowser::Chrome => {
                // Tests must run in headless mode without a
                // sandbox (required for Travis CI).
                new.always_match(
                    "goog:chromeOptions", json!({
                        "args": ["--no-sandbox", "--headless"],
                    }))
            }
        };

        new
    }
}

/// Tests defined in this macro are run once per browser. See the macro invocations below.
macro_rules! browser_tests {
    ($mod_name:ident, $browser_type:expr) => {
        mod $mod_name {
            use super::*;
            fn test_browser() -> TestBrowser { $browser_type }

            #[test]
            fn navigation() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                assert_eq!(&sess.get_current_url().expect("Error getting url [1]"), &page1, "Wrong URL [1]");

                let page2 = server.url("/page2.html");
                sess.go(&page2).expect("Error going to page2");
                assert_eq!(&sess.get_current_url().expect("Error getting url [2]"), &page2, "Wrong URL [2]");

                sess.back().expect("Error going back");
                assert_eq!(&sess.get_current_url().expect("Error getting url [3]"), &page1, "Wrong URL [3]");

                sess.forward().expect("Error going forward");
                assert_eq!(&sess.get_current_url().expect("Error getting url [4]"), &page2, "Wrong URL [4]");
            }

            #[test]
            fn title() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                assert_eq!(&sess.get_title().expect("Error getting title"), "Test page 1 title", "Wrong title");
            }

            #[test]
            fn get_page_source() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let page_source = sess.get_page_source().expect("Error getting page source");

                if sess.browser_name() == Some("chrome") {
                    // chrome sets the xmlns attribute in the html element
                    assert!(page_source.contains(r#"<html>"#), "Want page_source to contain <html> but was {}", page_source);
                } else {
                    assert!(page_source.contains("<html>"), "Want page_source to contain <html> but was {}", page_source);
                }
                assert!(page_source.contains("<title>Test page 1 title</title>"), "Want page_source to contain <title>Test page 1 title</title> but was {}", page_source);
            }

            #[test]
            fn find_element_by_css() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("span.red", LocationStrategy::Css).expect("Error finding element");
                assert_eq!(element.text().expect("Error getting text"), "Red text", "Wrong element found");

                sess.find_element("body.red", LocationStrategy::Css).expect_err("Want error");
            }

            #[test]
            fn find_element_by_link_text() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("A really handy WebDriver crate", LocationStrategy::LinkText).expect("Error finding element");
                assert_eq!(element.text().expect("Error getting text"), "A really handy WebDriver crate", "Wrong element found");

                sess.find_element("A link with this text does not appear on the page", LocationStrategy::LinkText).expect_err("Want error");
            }

            #[test]
            fn find_element_by_partial_link_text() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("crate", LocationStrategy::PartialLinkText).expect("Error finding element");
                assert_eq!(element.text().expect("Error getting text"), "A really handy WebDriver crate", "Wrong element found");

                sess.find_element("A link with this text does not appear on the page", LocationStrategy::PartialLinkText).expect_err("Want error");
            }

            #[test]
            fn find_element_by_xpath() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("//a", LocationStrategy::XPath).expect("Error finding element");
                assert_eq!(element.text().expect("Error getting text"), "A really handy WebDriver crate", "Wrong element found");

                sess.find_element("//video", LocationStrategy::XPath).expect_err("Want error");
            }

            #[test]
            fn find_elements_by_css() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let elements = sess.find_elements("span.red", LocationStrategy::Css).expect("Error finding elements");
                let element_texts: Vec<String> = elements.into_iter().map(|elem| elem.text().expect("Error getting text")).collect();
                assert_eq!(element_texts, vec!["Red text".to_owned(), "More red text".to_owned()], "Wrong element texts");

                let found_elements = sess.find_elements("body.red", LocationStrategy::Css).expect("Error finding absent elements");
                assert!(found_elements.is_empty(), "Want to find no elements, found {:?}", found_elements);
            }

            #[test]
            fn find_elements_by_link_text() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let elements = sess.find_elements("A really handy WebDriver crate", LocationStrategy::LinkText).expect("Error finding elements");
                let element_texts: Vec<String> = elements.into_iter().map(|elem| elem.text().expect("Error getting text")).collect();
                assert_eq!(element_texts, vec!["A really handy WebDriver crate".to_owned()], "Wrong element texts");

                let found_elements = sess.find_elements("A really bad WebDriver crate", LocationStrategy::LinkText).expect("Error finding absent elements");
                assert!(found_elements.is_empty(), "Want to find no elements, found {:?}", found_elements);
            }

            #[test]
            fn find_elements_by_partial_link_text() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let elements = sess.find_elements("crate", LocationStrategy::PartialLinkText).expect("Error finding elements");
                let element_texts: Vec<String> = elements.into_iter().map(|elem| elem.text().expect("Error getting text")).collect();
                assert_eq!(element_texts, vec!["A really handy WebDriver crate".to_owned(), "A WebDriver crate with just the server-side".to_owned()], "Wrong element texts");

                let found_elements = sess.find_elements("A really bad WebDriver crate", LocationStrategy::PartialLinkText).expect("Error finding absent elements");
                assert!(found_elements.is_empty(), "Want to find no elements, found {:?}", found_elements);
            }

            #[test]
            fn find_elements_by_xpath() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let elements = sess.find_elements("//body/span", LocationStrategy::XPath).expect("Error finding elements");
                let element_texts: Vec<String> = elements.into_iter().map(|elem| elem.text().expect("Error getting text")).collect();
                assert_eq!(element_texts, vec!["Red text".to_owned(), "More red text".to_owned()], "Wrong element texts");

                let found_elements = sess.find_elements("//video", LocationStrategy::XPath).expect("Error finding absent elements");
                assert!(found_elements.is_empty(), "Want to find no elements, found {:?}", found_elements);
            }

            #[test]
            fn element_attribute_and_property() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                let page2 = server.url("/page2.html");

                sess.go(&page1).expect("Error going to page1");
                let link = sess.find_element("#link_to_page_2", LocationStrategy::Css).expect("Error finding element");

                assert_eq!(&link.attribute("href").expect("Error getting attribute"),
                           "/page2.html");

                if sess.browser_name() == Some("chrome") {
                    // FIXME: chrome does not implement the property endpoint
                } else {
                    assert_eq!(&link.property("href").expect("Error getting property"), &page2);
                }
            }

            #[test]
            fn element_css_value() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("span.red", LocationStrategy::Css).expect("Error finding element");
                if sess.browser_name() == Some("chrome") {
                    assert_eq!(&element.css_value("color").expect("Error getting css value"), "rgba(255, 0, 0, 1)");
                } else {
                    assert_eq!(&element.css_value("color").expect("Error getting css value"), "rgb(255, 0, 0)");
                }
            }

            #[test]
            fn element_click() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let output = sess.find_element("#set-text-output", LocationStrategy::Css)
                                 .expect("Finding output element");
                assert_eq!(&output.text().expect("Getting output text"), "Unset");
                let button = sess.find_element("#set-text-btn", LocationStrategy::Css)
                                 .expect("Finding button element");
                button.click().expect("Click button");
                assert_eq!(&output.text().expect("Getting output text"), "Set");
            }

            #[test]
            fn element_clear() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("#textfield", LocationStrategy::Css).expect("Error finding element");
                assert_eq!(&element.property("value").expect("Error getting value [1]"), "Pre-filled");
                element.clear().expect("Error clearing element");
                assert_eq!(&element.property("value").expect("Error getting value [2]"), "");
            }

            #[test]
            fn element_send_keys() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("#textfield", LocationStrategy::Css).expect("Error finding element");
                assert_eq!(&element.property("value").expect("Error getting value [1]"),
                           "Pre-filled");
                element.send_keys(" hello").expect("Error sending keys to element");
                assert_eq!(&element.property("value").expect("Error getting value [2]"), "Pre-filled hello");
            }

            #[test]
            fn element_text() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("span.red", LocationStrategy::Css).expect("Error finding element");
                assert_eq!(&element.text().expect("Error getting text"), "Red text");
            }

            #[test]
            fn element_name() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("span.red", LocationStrategy::Css).expect("Error finding element");
                assert_eq!(&element.name().expect("Error getting name"), "span");
            }

            #[test]
            fn element_child() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("#parent", LocationStrategy::Css).expect("Error finding parent element");

                let child_by_css = element.find_element("span", LocationStrategy::Css).expect("Error finding child by CSS");
                assert_eq!(&child_by_css.attribute("id").expect("Error getting id [1]"), "child1");

                let child_by_xpath = element.find_element(".//span", LocationStrategy::XPath).expect("Error finding child by XPath");
                assert_eq!(&child_by_xpath.attribute("id").expect("Error getting id [2]"), "child1");
            }

            #[test]
            fn element_children() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let element = sess.find_element("#parent", LocationStrategy::Css).expect("Error finding parent element");

                let children = element.find_elements("span", LocationStrategy::Css).expect("Error finding children by CSS");
                assert_eq!(children.iter().map(|e| e.attribute("id").expect("Error getting id")).collect::<Vec<_>>(), vec!["child1".to_owned(), "child2".to_owned()]);
            }

            #[test]
            fn refresh() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let elem = sess.find_element("#textfield", LocationStrategy::Css).expect("Error finding element [1]");
                assert_eq!(elem.property("value").expect("Error getting value [1]"), "Pre-filled".to_owned());

                elem.clear().expect("Error clearing");
                assert_eq!(elem.property("value").expect("Error getting value [1]"), "".to_owned());

                sess.refresh().expect("Error refreshing");
                elem.text().expect_err("Want stale element error");
                let elem2 = sess.find_element("#textfield", LocationStrategy::Css).expect("Error finding element [1]");
                assert_eq!(elem2.property("value").expect("Error getting value [2]"), "Pre-filled".to_owned());
            }

            #[test]
            fn execute() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let exec_json = sess.execute(ExecuteCmd {
                    script: "return arguments[0] + arguments[1];".to_owned(),
                    args: vec![json!(1), json!(2)],
                }).expect("Error executing script");
                assert_eq!(serde_json::from_value::<i64>(exec_json).expect("Error converting result to i64"), 3);

                let exec_error = sess.execute(ExecuteCmd {
                    script: "throw 'foo';".to_owned(),
                    args: vec![],
                }).expect_err("Want error");

                // FIXME: Chrome represents errors differently
                if sess.browser_name() != Some("chrome") {
                    match exec_error {
                        webdriver_client::Error::WebDriverError(err) => assert!(format!("{:?}", err).contains("foo"), "Bad error message: {:?}", err),
                        other => panic!("Wrong error type: {:?}", other),
                    };
                }

            }

            #[test]
            fn browser_name() {
                let sess = test_browser().session();
                match test_browser() {
                    TestBrowser::Firefox => assert_eq!(sess.browser_name(), Some("firefox")),
                    TestBrowser::Chrome => assert_eq!(sess.browser_name(), Some("chrome")),
                }
            }

            #[test]
            fn execute_async() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let exec_json = sess.execute_async(ExecuteCmd {
                    script: "setTimeout(() => arguments[1](arguments[0]), 1000);".to_owned(),
                    args: vec![json!(1)],
                }).unwrap();
                let exec_int = serde_json::from_value::<i64>(exec_json).unwrap();
                assert_eq!(exec_int, 1);
            }

            // TODO: Test cookies

            // TODO: Test window handles

            #[test]
            fn frame_switch() {
                let (server, sess) = setup();
                let page3 = server.url("/page3.html");
                sess.go(&page3).expect("Error going to page3");

                // switching to parent from parent is harmless
                sess.switch_to_parent_frame().unwrap();

                let frames = sess.find_elements("iframe", LocationStrategy::Css).unwrap();
                assert_eq!(frames.len(), 1);

                sess.switch_to_frame(frames[0].reference().unwrap()).unwrap();
                let frames = sess.find_elements("iframe", LocationStrategy::Css).unwrap();
                assert_eq!(frames.len(), 2);

                for f in &frames {
                    sess.switch_to_frame(f.reference().unwrap()).unwrap();
                    let childframes = sess.find_elements("iframe", LocationStrategy::Css).unwrap();
                    assert_eq!(childframes.len(), 0);
                    sess.switch_to_parent_frame().unwrap();
                }

                sess.switch_to_parent_frame().unwrap();
                let frames = sess.find_elements("iframe", LocationStrategy::Css).unwrap();
                assert_eq!(frames.len(), 1);
            }

            #[test]
            fn http_driver() {
                ensure_logging_init();

                let driver = test_browser().driver();

                // Hackily sleep a bit until geckodriver is ready, otherwise our session
                // will fail to connect.
                // If this is unreliable, we could try:
                //   * Polling for the TCP port to become unavailable.
                //   * Wait for geckodriver to log "Listening on 127.0.0.1:4444".
                sleep(Duration::from_millis(1000));

                let http_driver = HttpDriverBuilder::default()
                                                    .url(driver.url())
                                                    .build().unwrap();

                let sess = http_driver.session(&test_browser().new_session_cmd())
                                      .unwrap();

                let server = FileServer::new();
                let test_url = server.url("/page1.html");
                sess.go(&test_url).unwrap();
                let url = sess.get_current_url().unwrap();
                assert_eq!(url, test_url);
            }

            #[test]
            fn screenshot_frame() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let ss = sess.screenshot().expect("Screenshot");
                std::fs::create_dir_all("target/screenshots").expect("Create screenshot dir");
                ss.save_file(&format!("target/screenshots/{:?}_frame.png",
                                      test_browser()))
                  .expect("Save screenshot");
            }

            #[test]
            fn screenshot_element() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let ss = sess.find_element("#parent", LocationStrategy::Css).expect("element")
                             .screenshot().expect("Screenshot");
                std::fs::create_dir_all("target/screenshots").expect("Create screenshot dir");
                ss.save_file(&format!("target/screenshots/{:?}_element.png",
                                      test_browser()))
                  .expect("Save screenshot");
            }

            #[test]
            fn dismiss_alert() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let btn = sess.find_element("#alert-btn", LocationStrategy::Css).expect("btn");
                btn.click().expect("click");
                sess.dismiss_alert().expect("dismiss alert");
            }

            #[test]
            fn accept_confirm_alert() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let btn = sess.find_element("#confirm-btn", LocationStrategy::Css)
                              .expect("find btn");
                btn.click().expect("click");
                sess.accept_alert().expect("accept alert");
                let out = sess.find_element("#alerts-out", LocationStrategy::Css)
                              .expect("find output");
                assert_eq!("true", out.text().expect("output text"));
            }

            #[test]
            fn dismiss_confirm_alert() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let btn = sess.find_element("#confirm-btn", LocationStrategy::Css)
                              .expect("find btn");
                btn.click().expect("click");
                sess.dismiss_alert().expect("accept alert");
                let out = sess.find_element("#alerts-out", LocationStrategy::Css)
                              .expect("find output");
                assert_eq!("false", out.text().expect("output text"));
            }

            #[test]
            fn get_alert_text() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let btn = sess.find_element("#alert-btn", LocationStrategy::Css)
                              .expect("find btn");
                btn.click().expect("click");
                assert_eq!("Alert", sess.get_alert_text().expect("get_alert_text"));
                sess.dismiss_alert().expect("accept alert");
            }

            #[test]
            fn send_alert_text() {
                let (server, sess) = setup();
                let page1 = server.url("/page1.html");
                sess.go(&page1).expect("Error going to page1");
                let btn = sess.find_element("#prompt-btn", LocationStrategy::Css)
                              .expect("find btn");
                btn.click().expect("click");
                sess.send_alert_text("foobar").expect("send_alert_text");
                sess.accept_alert().expect("accept alert");
                let out = sess.find_element("#alerts-out", LocationStrategy::Css)
                              .expect("find output");
                assert_eq!("foobar", out.text().expect("output text"));
            }

            fn setup() -> (FileServer, DriverSession) {
                ensure_logging_init();

                let session = test_browser().session();

                let server = FileServer::new();

                (server, session)
            }

            // End of browser_tests tests
        }
    }
}

browser_tests!(firefox, TestBrowser::Firefox);
browser_tests!(chrome, TestBrowser::Chrome);

fn ensure_logging_init() {
    static DONE: Once = Once::new();
    DONE.call_once(|| init_logging());
}

fn init_logging() {
    let mut builder = LogBuilder::new();
    builder.filter(None, LogLevelFilter::Info);
    builder.target(LogTarget::Stdout);

    if let Ok(ev) = env::var("RUST_LOG") {
       builder.parse(&ev);
    }

    builder.init().unwrap();
}

struct FileServer {
    listening: hyper::server::Listening,
    base_url: String,
}

impl FileServer {
    pub fn new() -> FileServer {
        for i in 0..2000 {
            let port = 8000 + i;
            let base_url = format!("http://localhost:{}", port);
            let server = match hyper::Server::http(("localhost", port)) {
                Ok(server) => server,
                Err(_) => {
                    continue;
                },
            };
            match server.handle_threads(FileServer::handle, 10) {
                Ok(listening) => {
                    return FileServer {
                        listening,
                        base_url,
                    };
                },
                Err(err) => panic!("Error listening: {:?}", err),
            }
        }
        panic!("Could not find free port to serve test pages")
    }

    pub fn url(&self, path: &str) -> String {
        format!("{base_url}{path}", base_url = self.base_url, path = path)
    }

    fn handle(req: hyper::server::Request, mut resp: hyper::server::Response) {
        match FileServer::handle_impl(&req) {
            Ok(bytes) => {
                *resp.status_mut() = hyper::status::StatusCode::Ok;
                resp.send(&bytes).expect("Failed to send HTTP response");
            },
            Err(err) => {
                eprintln!("{}", err);
                *resp.status_mut() = hyper::status::StatusCode::BadRequest;
            },
        };
    }

    fn handle_impl(req: &hyper::server::Request) -> Result<Vec<u8>, String> {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let www_root = crate_root.join("tests").join("www");

        match req.uri {
            hyper::uri::RequestUri::AbsolutePath(ref path) => {
                if path.starts_with("/") {
                    let abs_path = www_root.join(&path[1..]);
                    let file_path = std::fs::canonicalize(&abs_path);
                    match file_path {
                        Ok(realpath) => {
                            if realpath.starts_with(&www_root) {
                                let mut contents = Vec::new();
                                std::fs::File::open(&realpath)
                                    .and_then(|mut f| f.read_to_end(&mut contents))
                                    .map_err(|err| format!("Error reading file {:?}: {:?}", realpath, err))?;
                                return Ok(contents);
                            } else {
                                return Err(format!("Rejecting request for path outside of www: {:?}", realpath));
                            }
                        },
                        Err(err) => {
                            return Err(format!("Error canonicalizing file {:?}: {:?}", abs_path, err));
                        },

                    }
                } else {
                    return Err(format!("Received bad request for path {:?}", path));
                }
            },
            ref path => {
                return Err(format!("Received request for non-AbsolutePath: {:?}", path));
            },
        }
    }
}

impl Drop for FileServer {
    fn drop(&mut self) {
        self.listening.close().expect("FileServer failed to stop listening");
    }
}
