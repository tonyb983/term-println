#![feature(round_char_boundary)]
#![allow(dead_code, unused)]

mod fmt;
mod help;

use std::{env, sync::atomic::AtomicBool};

pub use fmt::*;

static PRINT_DEBUG: AtomicBool = AtomicBool::new(false);

fn main() -> Result<()> {
    let bin = env::args().next().expect("Unable to get env::args[0]");
    let mut all_args = env::args().skip(1).collect::<Vec<_>>();
    match all_args.len() {
        0 => help::print_usage(&bin),
        1 => {
            if &all_args[0] == "--help" {
                help::print_usage_long(&bin)
            } else if &all_args[0] == "-h" {
                help::print_usage(&bin)
            } else {
                print_string(&all_args[0])
            }
        }
        _ => {
            if &all_args[0] == "--debug" || &all_args[0] == "-d" || &all_args[0] == "-D" {
                PRINT_DEBUG.store(true, std::sync::atomic::Ordering::Relaxed);
                all_args.remove(0);
            }
            format(&bin, &all_args)
        }
    }
}

fn format<S: std::fmt::Display>(bin: &str, all_args: &[S]) -> Result<()> {
    let input_len = all_args.len();
    if input_len == 0 {
        return help::print_usage(bin);
    } else if input_len == 1 {
        return print_string(&all_args[0]);
    }

    let f = fmt::Formatter::new(&all_args[0].to_string())?;
    if PRINT_DEBUG.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Formatter: {:#?}", f);
    }
    let output = f.generate(&all_args[1..])?;
    println!("{}", output);

    Ok(())
}

fn print_string<S: std::fmt::Display>(s: S) -> Result<()> {
    println!("{}", s);
    Ok(())
}
