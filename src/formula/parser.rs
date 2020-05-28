use super::*;
use std::fs::File;
use std::io::{Read, Cursor, BufRead};

type ParseResult = std::result::Result<Formula, String>;

impl Formula {
    pub fn parse_dimacs(mut file: File) -> ParseResult {
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|_| "Error while reading file")?;
        let mut buf = Cursor::new(buf);

        let problem_line: String = (&mut buf).lines()
            .map(Result::unwrap)
            .find(|l| !l.starts_with('c'))
            .ok_or("Missing problem line")?;

        let num_vars: usize;
        let num_clauses: usize;
        let params: Vec<_> = problem_line.split_whitespace().collect();
        if params.len() != 4 {
            return Err("Wrong number of parameters in problem line".to_owned());
        } else if params[0] != "p" {
            return Err("Invalid problem line".to_owned());
        } else if params[1] != "cnf" {
            return Err("Only cnf-formatted inputs are currently allowed".to_owned());
        } else {
            num_vars = params[2].parse()
                .map_err(|_| "Third problem line parameter invalid".to_owned())?;
            num_clauses = params[3].parse()
                .map_err(|_| "Fourth problem line parameter invalid".to_owned())?;
        }

        let mut formula = Formula {
            clauses: Vec::with_capacity(num_clauses),
            assignment: Assignment::new(num_vars),
            clause_indices: vec![vec![]; num_vars],
            remaining_clauses: num_clauses,
            unsolvable: false,
            next_literal_id: 0
        };

        let pos = buf.position() as usize;
        let buf = &buf.into_inner()[pos..];
        let mut clause_iter = buf.trim_end().split(" 0");
        for (clause, clause_str) in (&mut clause_iter).take(num_clauses).enumerate() {
            formula.clauses.push((Clause::new(), false));
            for v in clause_str.split_whitespace() {
                let v: isize = v.parse().map_err(|_| format!("Illegal variable '{}'", v))?;
                let lit = Literal::from_var(v);
                formula.clauses[clause].0.add(lit);
                formula.clause_indices[lit.id].push((clause, lit.negated));
            }
        }

        match clause_iter.next() {
            Some("") => Ok(formula),
            None => Err("Not enough clauses".to_owned()),
            _ => Err("Too many clauses".to_owned()),
        }
    }
}