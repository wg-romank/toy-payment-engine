use std::env::args;

mod account;
mod csv;
mod engine;
mod types;

use crate::csv::{dump_state, process_csv};
use engine::Engine;

fn main() {
    if let Some(path) = args().nth(1) {
        let mut engine = Engine::empty();

        process_csv(&mut engine, &path).expect("failed processing transactions");

        dump_state(&engine).expect("failed saving data");
    }
}
