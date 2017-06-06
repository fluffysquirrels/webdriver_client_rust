#![deny(warnings)]

extern crate env_logger;
extern crate log;
extern crate serde_json;
extern crate webdriver_client;

use env_logger::{LogBuilder, LogTarget};
use log::LogLevelFilter;
use std::env;
use std::sync::{Once, ONCE_INIT};
use std::thread::sleep;
use std::time::Duration;
use webdriver_client::{Driver, HttpDriverBuilder};
use webdriver_client::firefox::GeckoDriver;
use webdriver_client::messages::ExecuteCmd;

#[test]
fn test_file() {
    ensure_logging_init();

    // TODO: Perhaps calculate path from PATH environment variable.
    let gecko = GeckoDriver::build()
        .firefox_binary("/usr/bin/firefox")
        .spawn().unwrap();
    let sess = gecko.session().unwrap();

    let test_url = get_integration_test_url();
    sess.go(&test_url).unwrap();
    let url = sess.get_current_url().unwrap();
    assert_eq!(url, test_url);

    let title = sess.get_title().unwrap();
    assert_eq!(title, "Test page title");

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

    // TODO: Perhaps calculate path from PATH environment variable.
    let gecko = GeckoDriver::build()
        .firefox_binary("/usr/bin/firefox")
        .spawn().unwrap();

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

    let test_url = get_integration_test_url();
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

fn get_integration_test_url() -> String {
    // `cargo test` starts test binary with current directory set to
    // the crate root.
    let crate_root =
        std::env::current_dir().unwrap()
        .to_str().unwrap().to_owned();
    format!("file://{crate}/tests/integration_test.html", crate = crate_root)
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
