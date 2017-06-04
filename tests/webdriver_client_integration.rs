#![deny(warnings)]

extern crate env_logger;
extern crate log;
extern crate serde_json;
extern crate webdriver;

use env_logger::LogBuilder;
use log::LogLevelFilter;
use std::env;
use webdriver::DriverSession;
use webdriver::firefox::GeckoDriver;
use webdriver::messages::ExecuteCmd;

#[test]
fn test_file() {
    init_logging();

    let gecko = GeckoDriver::new().unwrap();
    let mut sess = DriverSession::new(gecko).unwrap();

    // `cargo test` starts with current directory set to the crate root.
    let crate_root =
        std::env::current_dir().unwrap()
        .to_str().unwrap().to_owned();
    let test_url = format!("file://{crate}/tests/integration_test.html", crate = crate_root);

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
            webdriver::Error::WebDriverError(e) => e,
            _ => panic!("Unexpected error variant: {:#?}", err),
        };
        assert_eq!(err.message, "SomeException");
    }

    // TODO(alex): Test execute_async() success / failure.

    sess.close_window().unwrap();
}

fn init_logging() {
    let mut builder = LogBuilder::new();
    builder.filter(None, LogLevelFilter::Info);

    if let Ok(ev) = env::var("RUST_LOG") {
       builder.parse(&ev);
    }

    builder.init().unwrap();
}

mod youtube_integration_test {
    use webdriver::firefox::GeckoDriver;
    use webdriver::messages::LocationStrategy;
    use webdriver::DriverSession;

    #[test]
    #[ignore]
    fn test() {
        let gecko = GeckoDriver::new().unwrap();
        let mut sess = DriverSession::new(gecko).unwrap();
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
