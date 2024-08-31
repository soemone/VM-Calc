mod tokens;
mod lexer;
mod parser;
mod bytecode;
mod instruction;
mod ast;
mod errors;
mod utils;
mod vm;
mod functions;
mod processchain;

// Tests
mod tests;

use processchain::ProcessChain;

fn main() -> Result<(), ()> {
    // run()?;
    let src = "<id";
    let mut lexer = lexer::Lexer::new(src).unwrap();
    loop {
        match lexer.next() {
            Ok(result) => println!("OK -> {result:?}"),
            Err(errors::Error::TEOF) => break,
            Err(error) => {println!("{error}");},
        }
    }
    Ok(())
}

fn run() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().collect();
    match args[1].as_str() {
        "-rb" | "--run-binary" => ProcessChain::run_from_bytecode(&args[2])?,
        "-wb" | "--write-binary" => ProcessChain::store_bytecode_from_file(&args[2], &args[3])?,
        "-rf" | "--run-file" => ProcessChain::run_from_file(&args[2])?,
        "-rfs" | "--run-store" | "--run-and-store-file" => {
            let output = 
                if args.len() >= 4 { &args[3] } 
                else { 
                    let location = args[2].rfind(".");
                    let mut res = args[2].as_str();
                    res = match location {
                        Some(idx) => &res[0..idx],
                        None => res,
                    };
                    &format!("{}.bin", res) 
                };
            ProcessChain::store_bytecode_from_file(&args[2], output)?;
            ProcessChain::run_from_file(&args[2])?;
        },
        "-t" | "--text" => ProcessChain::run_from_text(&args[2])?,
        arg => println!("Invalid argument `{}` provided.", arg)
    }
    Ok(())
}