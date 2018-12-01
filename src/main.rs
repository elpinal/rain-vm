extern crate structopt;

use structopt::StructOpt;

use rain_vm::vm::execute_file;

macro_rules! exitln {
    ( $code:expr, $($x:expr),* ) => {
        {
            eprintln!($($x),*);
            ::std::process::exit($code);
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "rain-vm", author = "", version_short = "v")]
/// Rain VM.
struct Opt {
    /// Input filename
    #[structopt(name = "filename")]
    file: String,
}

fn main() {
    let opt = Opt::from_args();
    match execute_file(&opt.file) {
        Ok(b) => println!("{}", b),
        Err(e) => exitln!(1, "{}", e),
    }
}
