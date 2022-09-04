use std::env::args;

mod types;
mod account;
mod engine;
mod csv;

use engine::Engine;
use crate::csv::{process_csv, dump_state};

fn main() {
    if let Some(path) = args().nth(1) {
        let mut engine = Engine::empty();

        process_csv(&mut engine, &path)
            .expect("failed processing transactions");

        dump_state(&engine)
            .expect("failed saving data");
    }
}
