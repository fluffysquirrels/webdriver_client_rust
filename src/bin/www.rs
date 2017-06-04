extern crate webdriver_client;
use webdriver_client::*;
use webdriver_client::messages::{LocationStrategy, ExecuteCmd};
use webdriver_client::firefox::GeckoDriver;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

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
    let gecko = GeckoDriver::spawn()
        .expect("Unable to start geckodriver");
    let sess = gecko.session()
        .expect("Unable to start WebDriver session");

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
