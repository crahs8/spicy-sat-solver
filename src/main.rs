#![feature(vec_remove_item)]

use std::env;
use std::fs::File;
use solver::Formula;

mod solver;

fn main() {
    let file_name = env::args().nth(1)
        .expect("Please provide an input file");
    let file = File::open(file_name)
        .expect("Failed to open file");

    let formula = Formula::parse_dimacs(file)
        .unwrap_or_else(|msg| panic!("Couldn't parse file: {}", msg));
    match formula.solve() {
        Some(a) => println!("{}", a),
        None => println!("UNSATISFIABLE"),
    }
}
