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
            assign_history: vec![],
            remaining_clauses: num_clauses,
            unsolvable: false,
            next_literal_id: 0
        };

        let pos = buf.position() as usize;
        let buf = &buf.into_inner()[pos..];
        let mut clause_iter = buf.trim_end().split(" 0");

        'outer: for (clause_idx, clause_str) in (&mut clause_iter).take(num_clauses).enumerate() {
            let mut clause = Clause::new();

            for v in clause_str.split_whitespace() {
                let v: isize = v.parse().map_err(|_| format!("Illegal variable '{}'", v))?;
                let lit = Literal::from_var(v);
                // Check if we have a | !a, which we rely upon not existing in the solver
                if clause.0.contains(&!lit) {
                    formula.remaining_clauses -= 1;
                    continue 'outer;
                }
                clause.add(lit);
            }

            for lit in &clause.0 {
                formula.clause_indices[lit.id].push((clause_idx, lit.negated));
            }
            formula.clauses.push((clause, false));
        }

        match clause_iter.next() {
            Some("") => Ok(formula.simplify()),
            None => Err("Not enough clauses".to_owned()),
            _ => Err("Too many clauses".to_owned()),
        }
    }

    /// Unit propagation
    fn simplify(mut self) -> Self {
        for i in 0..self.clauses.len() {
            if let Some(l) = self.clauses[i].0.get_unit_literal() {
                self.assign(l);
            }
        }

        self
    }
}