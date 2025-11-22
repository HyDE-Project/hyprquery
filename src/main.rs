mod app;
mod cli;
mod error;
mod export;
mod path;
mod query;
mod schema;
mod source;
mod value;

use std::process;

fn main() {
    match app::run() {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
