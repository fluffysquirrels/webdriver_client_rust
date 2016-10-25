
extern crate webdriver;
use webdriver::*;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn execute_function<T>(name: &str, args: &str, sess: &DriverSession<T>) -> Result<(), Error> {
    match name {
        "back" => try!(sess.back()),
        "go" => try!(sess.go(args)),
        "refresh" => try!(sess.refresh()),
        "source" => println!("{}", try!(sess.get_page_source())),
        "url" => println!("{}", try!(sess.get_current_url())),
        _ => println!("Unknown function: {}", name),
    }
    Ok(())
}

fn execute<T>(line: &str, sess: &DriverSession<T>) -> Result<(), Error>{
    let (cmd, args) = line.find(' ')
        .map_or((line, "".as_ref()), |idx| line.split_at(idx));
    execute_function(cmd, args, sess)
}

fn main() {
    let gecko = GeckoDriver::new()
        .expect("Unable to start geckodriver");
    let sess = DriverSession::new(gecko)
        .expect("Unable to start WebDriver session");

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                if let Err(err) = execute(&line, &sess) {
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
