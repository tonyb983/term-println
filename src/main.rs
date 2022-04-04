#![allow(dead_code, unused)]

mod fmt;

use std::env;

pub use fmt::{Error, Result};

fn main() -> Result<()> {
    let bin = env::args().next().expect("Unable to get env::args[0]");
    let all_args = env::args().skip(1).collect::<Vec<_>>();
    match all_args.len() {
        0 => print_usage(&bin),
        1 => {
            if &all_args[0] == "--help" || &all_args[0] == "-h" {
                print_usage(&bin)
            } else {
                print_string(&all_args[0])
            }
        }
        _ => format(&bin, &all_args),
    }
}

fn format<S: std::fmt::Display>(bin: &str, all_args: &[S]) -> Result<()> {
    let input_len = all_args.len();
    if input_len == 0 {
        return print_usage(bin);
    } else if input_len == 1 {
        return print_string(&all_args[0]);
    }

    let f = fmt::Formatter::new(&all_args[0].to_string())?;
    let output = f.generate(&all_args[1..])?;
    println!("{}", output);

    Ok(())
}

fn print_string<S: std::fmt::Display>(s: S) -> Result<()> {
    println!("{}", s);
    Ok(())
}

fn print_usage(bin: &str) -> Result<()> {
    let trimmed = if let Some(n) = bin.rfind(['/', '\\']) {
        &bin[n + 1..]
    } else {
        bin
    };
    println!("Usage:");
    println!("\t$ {} <FMT_STRING> [<ARGS>]", trimmed);
    println!("FMT_STRING - A string containing text and any number of format specifiers");
    println!("ARGS - A list of strings to be inserted into the FMT_STRING");
    println!();
    Ok(())
}
