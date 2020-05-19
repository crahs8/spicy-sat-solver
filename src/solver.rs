use std::fs::File;
use std::fmt;
use std::io::{BufReader, BufRead};
use std::ops::Not;
use fnv::FnvHashSet;

/// A literal consisting of a variable id and whether that variable is negated
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Literal {
    var_id: usize,
    negated: bool,
}

impl Literal {
    fn new(var: i32) -> Self {
        Literal {
            var_id: var.abs() as usize,
            negated: var < 0,
        }
    }
}

impl Not for Literal {
    type Output = Literal;

    fn not(self) -> Self::Output {
        Literal {
            var_id: self.var_id,
            negated: !self.negated,
        }
    }
}

type ParseResult = std::result::Result<Formula, String>;

/// A Formula is a set of disjunctions (which are sets of literals)
pub struct Formula {
    formula: Vec<Vec<Literal>>,
    removed: FnvHashSet<*const Literal>,
    num_vars: usize,
}

impl Formula {
    /// Parses a DIMACS file and returns the corresponding formula or an error
    pub fn parse_dimacs(file: File) -> ParseResult {
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Parse comments and problem line
        let mut num_vars: usize = 0;
        let mut num_clauses: usize = 0;
        for line in (&mut lines).map(|l| l.unwrap()) {
            // Skip comments
            if line.starts_with('c') { continue; }
            // Handle problem line
            else if line.starts_with('p') {
                let args: Vec<_> = line.split_whitespace().skip(1).collect();
                if args.len() != 3 || args[0] != "cnf" {
                    return Err("Illegal problem line".to_owned());
                }
                num_vars = args[1].parse()
                    .map_err(|_| "2nd argument in problem line not valid".to_owned())?;
                num_clauses = args[2].parse()
                    .map_err(|_| "3rd argument in problem line not valid".to_owned())?;
                break;
            } else {
                return Err("Illegal problem line".to_owned());
            }
        }
        if num_vars == 0 || num_clauses == 0 {
            return Err("Illegal problem line".to_owned())
        }

        // Parse the variables
        let mut formula = Formula {
            formula: vec![Vec::new(); num_clauses],
            removed: FnvHashSet::with_capacity_and_hasher(num_clauses, Default::default()),
            num_vars,
        };
        let vars: String = lines.map(|l| l.unwrap() + " ").collect();
        let mut clause = 0;
        for var in vars.split_whitespace().map(|v| v.parse::<i32>()) {
            let var = var.map_err(|e| format!("Illegal variable: {}", e))?;
            if var == 0 {
                clause += 1;
            } else {
                formula.formula.get_mut(clause).ok_or("Too many clauses")?
                    .push(Literal::new(var));
            }
        }

        if clause != num_clauses {
            Err("Too few clauses".to_owned())
        } else {
            Ok(formula)
        }
    }

    fn simplify(&mut self) {
        // Unit propagation
        // TODO: Bench vs repeated loops
        let mut changed = false;
        let mut unit_lits = vec![];
        for clause in self.formula.iter().filter(|c| !self.removed.contains(&c.as_ptr())) {
            if clause.len() == 1 {
                changed = true;
                let lit = unsafe { clause.get_unchecked(0) };
                unit_lits.push(*lit);
            }
        }
        for lit in unit_lits {
            'clauses: for clause in self.formula.iter_mut() {
                if self.removed.contains(&clause.as_ptr()) {
                    continue;
                }
                for l in clause.iter() {
                    if *l == lit {
                        self.removed.insert(clause.as_ptr());
                        continue 'clauses;
                    }
                }
                clause.remove_item(&!lit);
            }
        }
        if changed {
            self.simplify();
        }
    }

    fn solved(&self, assignment: &Assignment) -> bool {
        'outer: for disjunction in self.formula.iter().filter(|c| !self.removed.contains(&c.as_ptr())) {
            for lit in disjunction {
                if assignment.is_assigned(*lit) {
                    continue 'outer;
                }
            }
            return false;
        }
        true
    }

    fn unsolvable(&self, assignment: &Assignment) -> bool {
        'outer: for disjunction in self.formula.iter().filter(|c| !self.removed.contains(&c.as_ptr())) {
            for lit in disjunction {
                if assignment.is_not_negated(*lit) {
                    continue 'outer;
                }
            }
            return true;
        }
        false
    }

    /// Solves the formula and returns an Assignment or None if it isn't possible
    pub fn solve(mut self) -> Option<Assignment> {
        let mut assignment = Assignment::new(self.num_vars);

        fn dpll(formula: &mut Formula, assignment: &mut Assignment) -> bool {
            //formula.simplify();
            if formula.solved(&assignment) {
                true
            } else if formula.unsolvable(&assignment) {
                false
            } else {
                let next = assignment.next_literal();
                assignment.assign(next);
                if dpll(formula, assignment) {
                    true
                } else {
                    assignment.un_assign(next);
                    assignment.assign(!next);
                    if dpll(formula, assignment) {
                        true
                    } else {
                        assignment.un_assign(next);
                        false
                    }
                }
            }
        }

        if dpll(&mut self, &mut assignment) {
            Some(assignment)
        } else {
            None
        }
    }
}

/// An assignment of literals
/// Each element in the `Vec` corresponds to a literal except for the first
/// Elements may be None, corresponding to no assignment
pub struct Assignment {
    assignment: Vec<Option<bool>>,
    num_assigned: usize,
}

impl Assignment {
    fn new(num_vars: usize) -> Self {
        Assignment {
            assignment: vec![None; num_vars + 1],
            num_assigned: 0,
        }
    }
    /// Returns true if the given literal (also respecting negation) is assigned and false otherwise
    fn is_assigned(&self, lit: Literal) -> bool {
        Some(lit.negated) == self.assignment[lit.var_id]
    }
    /// Returns false if the negation of the given literal is assigned and true otherwise
    fn is_not_negated(&self, lit: Literal) -> bool {
        Some(!lit.negated) != self.assignment[lit.var_id]
    }
    fn next_literal(&self) -> Literal {
        Literal {
            var_id: self.num_assigned + 1,
            negated: false,
        }
    }
    fn assign(&mut self, lit: Literal) {
        self.assignment[lit.var_id] = Some(lit.negated);
        self.num_assigned += 1;
    }
    fn un_assign(&mut self, lit: Literal) {
        self.assignment[lit.var_id] = None;
        self.num_assigned -= 1;
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (var_id, negated) in self.assignment.iter().enumerate().skip(1) {
            match negated {
                Some(n) => write!(f, "{}{} ", if *n { "-" } else { "" }, var_id)?,
                None => write!(f, "{} UNASSIGNED", var_id)?,
            }

        }
        write!(f, "0")
    }
}