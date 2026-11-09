// Copyright (c) <OWNER>

mod error;
mod bytes;
mod loader;
mod decoder;
mod ir;
mod humanizer;
mod analyzer;
mod emulator;
mod trace;
mod diff;
mod report;
mod dashboard;

use std::env;
use std::process::exit;

use kashshaf::error::Result;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: kashshaf <binary>");
        exit(2);
    }
    let _path = &args[1];
    // CLI wired in later phases; Phase 1 exposes the library API.
    let _: Result<()> = Ok(());
    exit(0);
}
