use std::fs::File;
use std::fmt;
use std::ops::Not;
use std::io::{Read, Cursor, BufRead};

/// A set of clauses
pub struct Formula {
    clauses: Vec<Clause>,
    assignment: Assignment,
    clause_indices: Vec<Vec<(usize, bool)>>,
    assign_history: Vec<Vec<Literal>>,
    next_literal_id: usize,
}

impl Formula {
    /// Parses a DIMACS file and returns the corresponding formula or an error
    pub fn parse_dimacs(mut file: File) -> Result<Formula, String> {
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|_| "Error while reading file")?;
        let mut buf = Cursor::new(buf);

        // Parse comments and problem line
        let problem_line: String = (&mut buf).lines()
            .map(Result::unwrap)
            .find(|l| !l.starts_with('c'))
            .ok_or("Missing problem line")?;

        let num_vars: usize;
        let num_clauses: usize;
        let params: Vec<_> = problem_line.split_whitespace().collect();
        if params.len() != 4 || params[0] != "p" {
            return Err("Invalid/Missing problem line".to_owned());
        } else if params[1] != "cnf" {
            return Err("Only cnf-formatted inputs are currently supported".to_owned());
        } else {
            num_vars = params[2].parse()
                .map_err(|_| "Third problem line parameter invalid".to_owned())?;
            num_clauses = params[3].parse()
                .map_err(|_| "Fourth problem line parameter invalid".to_owned())?;
        }

        // Parse the variables
        let mut formula = Formula {
            clauses: Vec::with_capacity(num_clauses),
            assignment: Assignment::new(num_vars),
            clause_indices: vec![vec![]; num_vars],
            assign_history: vec![],
            next_literal_id: 0,
        };

        let pos = buf.position() as usize;
        let buf = &buf.into_inner()[pos..];
        let mut clause_str_iter = buf.trim_end().split(" 0");

        'outer: for (clause_idx, clause_str) in (&mut clause_str_iter).take(num_clauses).enumerate() {
            let mut clause = Clause::new();
            for v in clause_str.split_whitespace() {
                let v: isize = v.parse().map_err(|_| format!("Illegal variable '{}'", v))?;
                let lit = Literal::from_var(v);
                // We rely upon a | !a not being present
                if clause.0.contains(&!lit) {
                    continue 'outer;
                }
                clause.0.push(lit);
                formula.clause_indices[lit.id].push((clause_idx, lit.negated));
            }
            formula.clauses.push(clause);
        }

        match clause_str_iter.next() {
            Some("") => Ok(formula),
            None => Err("Not enough clauses".to_owned()),
            _ => Err("Too many clauses".to_owned()),
        }
    }

    fn solved(&self) -> bool {
        self.clauses.iter().all(|c| c.solved(&self.assignment))
    }

    fn unsolvable(&self) -> bool {
        self.clauses.iter().any(|c| c.unsolvable(&self.assignment))
    }

    fn next_un_assigned(&self) -> Literal {
        let id = (self.next_literal_id..)
            .find(|&i| self.assignment.0[i].is_none()).unwrap();
        Literal {
            id,
            negated: false,
        }
    }

    fn assign(&mut self, lit: Literal) {
        self.next_literal_id = lit.id + 1;
        self.assign_history.push(vec![]);
        fn inner(formula: &mut Formula, lit: Literal) {
            formula.assignment.assign(lit);
            formula.assign_history.last_mut().unwrap().push(lit);
            for idx in 0..formula.clause_indices[lit.id].len() {
                let (clause_idx, negated) = formula.clause_indices[lit.id][idx];
                let clause = &formula.clauses[clause_idx];
                if negated != lit.negated {
                    let (count, unit_lit) = clause.0.iter()
                        .filter(|l| !formula.assignment.id_assigned(l.id))
                        .fold((0, Literal::default()), |(n, _), &l| (n + 1, l));
                    if count == 1 {
                        inner(formula, unit_lit);
                    }
                }
            }
        }
        inner(self, lit);
    }

    fn un_assign(&mut self, lit: Literal) {
        self.next_literal_id = lit.id;
        for lit in self.assign_history.pop().unwrap() {
            self.assignment.un_assign(lit);
        }
    }

    /// Solves the formula and returns an Assignment or None if it isn't possible
    pub fn solve(mut self) -> Option<Assignment> {
        if self.dpll() {
            Some(self.assignment)
        } else {
            None
        }
    }

    fn dpll(&mut self) -> bool {
        !self.unsolvable() && (self.solved() || {
            let next = self.next_un_assigned();
            self.assign(next);
            self.dpll() || {
                self.un_assign(next);
                self.assign(!next);
                let res = self.dpll();
                if !res { self.un_assign(!next) }
                res
            }
        })
    }
}

/// A disjunction of literals
#[derive(Clone, Debug)]
struct Clause(Vec<Literal>);

impl Clause {
    fn new() -> Self {
        Clause(vec![])
    }

    fn solved(&self, assignment: &Assignment) -> bool {
        self.0.iter().any(|l| assignment.assigned(*l))
    }

    fn unsolvable(&self, assignment: &Assignment) -> bool {
        self.0.iter().all(|l| assignment.assigned(!*l))
    }
}

/// A propositional variable (p, q, etc.) with some id which may be negated
/// Ex.: p, !q
#[derive(Copy, Clone, PartialEq, Default, Debug)]
struct Literal {
    id: usize,
    negated: bool,
}

impl Literal {
    // Creates a literal from a variable. A variable is e.g. 3 or -42,
    // which would have ids of 2 and 41 respectively
    fn from_var(var: isize) -> Self {
        Literal {
            id: var.abs() as usize - 1,
            negated: var < 0,
        }
    }
}

impl Not for Literal {
    type Output = Literal;

    fn not(self) -> Self::Output {
        Literal {
            id: self.id,
            negated: !self.negated,
        }
    }
}

/// The assigned literals
/// Each spot in the Vec is either a bool determining whether the assigned literal is negated
/// or None, if neither literal with that id is assigned
pub struct Assignment(Vec<Option<bool>>);

impl Assignment {
    pub fn new(num_vars: usize) -> Self {
        Assignment(vec![None; num_vars])
    }

    fn assign(&mut self, lit: Literal) {
        self.0[lit.id] = Some(lit.negated);
    }

    fn un_assign(&mut self, lit: Literal) {
        self.0[lit.id] = None;
    }

    fn assigned(&self, lit: Literal) -> bool {
        self.0[lit.id] == Some(lit.negated)
    }
     fn id_assigned(&self, id: usize) -> bool {
         self.0[id] != None
     }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (id, negated) in self.0.iter().enumerate() {
            match negated {
                Some(n) => write!(f, "{}{} ", if *n { "-" } else { "" }, id)?,
                None => write!(f, "{} UNASSIGNED", id)?,
            }

        }
        write!(f, "0")
    }
}