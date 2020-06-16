use std::env;
use std::fs::File;
use solver::Formula;
use std::time::Instant;

mod solver;

fn main() -> Result<(), String> {
    let file_name = env::args().nth(1)
        .ok_or("Please provide an input file")?;
    let file = File::open(file_name)
        .map_err(|_| "Failed to open file")?;

    let start = Instant::now();

    let formula = Formula::parse_dimacs(file)?;
    match formula.solve() {
        Some(a) => println!("{}", a),
        None => println!("UNSATISFIABLE"),
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}