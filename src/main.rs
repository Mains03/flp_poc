use std::env;
use std::process;
use std::fs::File;
use std::io::{self, Read};

use crate::machine::translate::translate;
use cbpv::terms;

mod parser;
mod cbpv;
mod eval;
mod machine;

fn main() {
    let args: Vec<String> = env::args().collect();

     if args.len() < 1 || args.len() > 2 {
        eprintln!("Error: Expected one argument, but got {}.", args.len() - 1);
        eprintln!("Usage: {} source_file [number of solutions]", args[0]);
        process::exit(1);
    }

    let file_name = &args[1];

    let mut file = match File::open(file_name) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error: Could not open file '{}': {}", file_name, error);
            process::exit(1);
        }
    };

    let mut src = String::new();

    // Try to read the file contents
    match file.read_to_string(&mut src) {
        Ok(_) => { interpret(&mut src); }
        Err(error) => {
            eprintln!("Error: Could not read file '{}': {}", file_name, error);
            process::exit(1);
        }
    };
}

fn interpret(src: &mut String) {

    let ast = parser::parse(src).unwrap();
    let (main, env) = translate(ast);
    machine::eval(main, env.into());
}
