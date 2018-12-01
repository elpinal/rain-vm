use std::env;

use rain_vm::vm::execute_file;

macro_rules! exitln {
    ( $code:expr, $($x:expr),* ) => {
        {
            eprintln!($($x),*);
            ::std::process::exit($code);
        }
    }
}

fn main() {
    let mut args = env::args();
    args.next();
    let l = args.len();
    if l > 1 {
        exitln!(1, "too many arguments ({})", l);
    }
    match args.next() {
        None => exitln!(1, "missing argument"),
        Some(filename) => match execute_file(&filename) {
            Ok(b) => println!("{}", b),
            Err(e) => {
                exitln!(1, "{}", e);
            }
        },
    }
}
