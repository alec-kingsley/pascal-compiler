#![allow(dead_code)]

use std::env;
use std::fs;

mod ast;
mod definitions;
mod tokenizer;
mod x86_64_compiler;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Usage: cargo run -- src.pas dest.s");
    }
    // read source file
    let src = &args[1];

    let code = fs::read_to_string(src)
        .expect("Failed to read from file.");
    
    // compile program, get destination file
    let pascal_ast = ast::parse_program(&code);
    let (x86_64, errors, warnings) = x86_64_compiler::compile(pascal_ast, &code);
    
    if errors > 0 {
        print!("Compilation failed due to {} ", errors);
        if errors > 1 {
            print!("errors");
        } else {
            print!("error");
        }
        if warnings > 0 {
            print!(" and {} ", warnings);
            if warnings > 1 {
                println!("warnings.");
            } else {
                println!("warning.");
            }
        } else {
            println!(".");
        }
    } else if warnings > 0 {
        print!("Compiled with {} ", warnings);
        if warnings > 1 {
            println!("warnings.");
        } else {
            println!("warning.");
        }
    } else {
        println!("Compilation complete.");
    }
    if errors == 0 {
        let dest = &args[2];

        // write output and exit
        fs::write(dest,x86_64).expect("Failed to write to file.");
        println!("Successfully written to {}.",dest);
    }
}



