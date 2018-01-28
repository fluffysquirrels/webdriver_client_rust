extern crate webdriver_client;
use webdriver_client::*;
use webdriver_client::messages::{LocationStrategy, ExecuteCmd};
use webdriver_client::firefox::GeckoDriver;
use webdriver_client::chrome::ChromeDriver;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

extern crate clap;
use clap::{App, Arg};

extern crate stderrlog;

fn execute_function(name: &str, args: &str, sess: &DriverSession) -> Result<(), Error> {
    match name {
        "back" => try!(sess.back()),
        "go" => try!(sess.go(args)),
        "refresh" => try!(sess.refresh()),
        "source" => println!("{}", try!(sess.get_page_source())),
        "url" => println!("{}", try!(sess.get_current_url())),
        "innerhtml" => {
            for (idx, elem) in sess.find_elements(args, LocationStrategy::Css)?.iter().enumerate() {
                println!("#{} {}", idx, elem.inner_html()?);
            }
        }
        "outerhtml" => {
            for (idx, elem) in sess.find_elements(args, LocationStrategy::Css)?.iter().enumerate() {
                println!("#{} {}", idx, elem.outer_html()?);
            }
        }
        "windows" => {
            for (idx, handle) in sess.get_window_handles()?.iter().enumerate() {
                println!("#{} {}", idx, handle)
            }
        }
        "execute" => {
            let script = ExecuteCmd {
                script: args.to_owned(),
                args: vec![],
            };
            match sess.execute(script)? {
                JsonValue::String(ref s) => println!("{}", s),
                other => println!("{}", other),
            }
        }
        _ => println!("Unknown function: \"{}\"", name),
    }
    Ok(())
}

fn execute(line: &str, sess: &DriverSession) -> Result<(), Error>{
    let (cmd, args) = line.find(' ')
        .map_or((line, "".as_ref()), |idx| line.split_at(idx));
    execute_function(cmd, args, sess)
}

fn main() {
    let matches = App::new("www")
        .arg(Arg::with_name("attach-to")
             .help("Attach to a running webdriver")
             .value_name("URL")
             .takes_value(true))
        .arg(Arg::with_name("driver")
             .short("D")
             .long("driver")
             .possible_values(&["geckodriver", "chromedriver"])
             .default_value("geckodriver")
             .takes_value(true))
        .arg(Arg::with_name("verbose")
             .short("v")
             .multiple(true)
             .help("Increases verbose"))
        .get_matches();

    stderrlog::new()
        .module("webdriver_client")
        .verbosity(matches.occurrences_of("verbose") as usize)
        .init()
        .expect("Unable to initialize logging in stderr");

    let sess = match matches.value_of("attach-to") {
        Some(url) => HttpDriverBuilder::default()
            .url(url)
            .build().unwrap()
            .session()
            .expect("Unable to attach to WebDriver session"),
        None => match matches.value_of("driver").unwrap() {
            "geckodriver" => {
                GeckoDriver::spawn()
                    .expect("Unable to start geckodriver")
                    .session()
                    .expect("Unable to start Geckodriver session")
            }
            "chromedriver" => {
                ChromeDriver::spawn()
                    .expect("Unable to start chromedriver")
                    .session()
                    .expect("Unable to start chromedriver session")
            }
            unsupported => {
                // should be unreachable see Arg::possible_values()
                panic!("Unsupported driver: {}", unsupported);
            }
        }
    };

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                if let Err(err) = execute(line.trim_matches('\n'), &sess) {
                    println!("{}", err);
                }
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}
