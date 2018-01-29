#![deny(warnings)]

extern crate env_logger;
extern crate hyper;
extern crate log;
extern crate serde_json;
extern crate webdriver_client;

use env_logger::{LogBuilder, LogTarget};
use log::LogLevelFilter;
use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Once, ONCE_INIT};
use std::thread::sleep;
use std::time::Duration;
use webdriver_client::{Driver, DriverSession, HttpDriverBuilder};
use webdriver_client::firefox::GeckoDriver;
use webdriver_client::messages::ExecuteCmd;

#[test]
fn test_file() {
    ensure_logging_init();

    let (server, sess) = setup();
    let test_url = server.url("/page1.html");
    sess.go(&test_url).unwrap();
    let url = sess.get_current_url().unwrap();
    assert_eq!(url, test_url);

    let title = sess.get_title().unwrap();
    assert_eq!(title, "Test page 1 title");

    sess.back().unwrap();
    sess.forward().unwrap();
    sess.refresh().unwrap();
    sess.get_page_source().unwrap();

    sess.get_cookies().unwrap();
    sess.get_window_handle().unwrap();
    {
        let handles = sess.get_window_handles().unwrap();
        assert_eq!(handles.len(), 1);
    }

    {
        // Test execute return
        let exec_json = sess.execute(ExecuteCmd {
            script: "return 2 + 2;".to_owned(),
            args: vec![],
        }).unwrap();
        let exec_int = serde_json::from_value::<i64>(exec_json).unwrap();
        assert_eq!(exec_int, 4);
    }

    {
        // Test execute handling an exception
        let exec_res = sess.execute(ExecuteCmd {
            script: "throw 'SomeException';".to_owned(),
            args: vec![],
        });
        assert!(exec_res.is_err());
        let err = exec_res.err().unwrap();
        let err = match err {
            webdriver_client::Error::WebDriverError(e) => e,
            _ => panic!("Unexpected error variant: {:#?}", err),
        };
        assert_eq!(err.error, "javascript error");
        assert_eq!(err.message, "SomeException");
    }

    {
        // Test execute async
        let exec_json = sess.execute_async(ExecuteCmd {
            script: "let resolve = arguments[0];\n\
                     setTimeout(() => resolve(4), 1000);".to_owned(),
            args: vec![],
        }).unwrap();
        let exec_int = serde_json::from_value::<i64>(exec_json).unwrap();
        assert_eq!(exec_int, 4);
    }

    // sess.close_window().unwrap();
}

#[test]
fn test_http_driver() {
    ensure_logging_init();

    let gecko = GeckoDriver::build().spawn().unwrap();

    // Hackily sleep a bit until geckodriver is ready, otherwise our session
    // will fail to connect.
    // If this is unreliable, we could try:
    //   * Polling for the TCP port to become unavailable.
    //   * Wait for geckodriver to log "Listening on 127.0.0.1:4444".
    sleep(Duration::from_millis(1000));

    let http_driver = HttpDriverBuilder::default()
                                        .url(gecko.url())
                                        .build().unwrap();
    let sess = http_driver.session().unwrap();

    let server = FileServer::new();
    let test_url = server.url("/page1.html");
    sess.go(&test_url).unwrap();
    let url = sess.get_current_url().unwrap();
    assert_eq!(url, test_url);
}

fn ensure_logging_init() {
    static DONE: Once = ONCE_INIT;
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

mod youtube_integration_test {
    use webdriver_client::Driver;
    use webdriver_client::firefox::GeckoDriver;
    use webdriver_client::messages::LocationStrategy;

    /// This depends on an external page not under our control, we
    /// should migrate to using local files.
    #[test]
    #[ignore]
    fn test() {
        let gecko = GeckoDriver::build()
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut sess = gecko.session().unwrap();
        sess.go("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        sess.get_current_url().unwrap();
        sess.back().unwrap();
        sess.forward().unwrap();
        sess.refresh().unwrap();
        sess.get_page_source().unwrap();

        {
            let el = sess.find_element("a", LocationStrategy::Css).unwrap();
            el.attribute("href").unwrap();
            el.css_value("color").unwrap();
            el.text().unwrap();
            assert_eq!(el.name().unwrap(), "a");

            let imgs = sess.find_elements("img", LocationStrategy::Css).unwrap();
            for img in &imgs {
                println!("{}", img.attribute("src").unwrap());
            }

            sess.get_cookies().unwrap();
            sess.get_title().unwrap();
            sess.get_window_handle().unwrap();
            let handles = sess.get_window_handles().unwrap();
            assert_eq!(handles.len(), 1);
        }
        sess.close_window().unwrap();
    }
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

fn setup() -> (FileServer, DriverSession) {
    ensure_logging_init();

    let gecko = GeckoDriver::build()
        .spawn().expect("Error starting geckodriver");
    let session = gecko.session().expect("Error starting session");

    let server = FileServer::new();

    (server, session)
}
