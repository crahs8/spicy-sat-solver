use super::*;
use std::fmt;

impl Formula {
    pub fn solve(mut self) -> Option<Assignment> {
        if self.dpll() {
            Some(self.assignment)
        } else {
            None
        }
    }

    /// The DPLL algorithm. Simplification happens on assignment
    fn dpll(&mut self) -> bool {
        if self.remaining_clauses == 0 {
            true
        } else if self.unsolvable {
            false
        } else {
            let next = self.next_un_assigned();
            self.assign(next);
            self.dpll() || {
                self.un_assign(next);
                self.assign(!next);
                let res = self.dpll();
                if !res { self.un_assign(!next) }
                res
            }
        }
    }

    fn next_un_assigned(&self) -> Literal {
        for id in self.next_literal_id.. {
            if self.assignment.0[id].is_none() {
                return Literal {
                    id,
                    negated: false,
                };
            }
        }
        unreachable!()
    }

    /// Assign a literal, performing unit propagation
    fn assign(&mut self, lit: Literal) {
        self.next_literal_id = lit.id + 1;
        self.assignment.assign(lit);
        for &(clause_idx, negated) in &self.clause_indices[lit.id] {
            if lit.negated != negated {
                let clause = &self.clauses[clause_idx].0;
                if clause.unsolvable(self) {
                    self.unsolvable = true;
                }
            } else if !self.clauses[clause_idx].1 {
                self.clauses[clause_idx].1 = true;
                self.remaining_clauses -= 1;
            }
        }
    }

    /// Un-assign a literal, undoing unit propagation
    fn un_assign(&mut self, lit: Literal) {
        self.next_literal_id = lit.id;
        self.assignment.un_assign(lit);
        for &(clause_idx, negated) in &self.clause_indices[lit.id] {
            if lit.negated != negated {
                self.unsolvable = false;
            } else {
                let (clause, removed) = &self.clauses[clause_idx];
                if *removed && !clause.solved(self) {
                    self.clauses[clause_idx].1 = false;
                    self.remaining_clauses += 1;
                }
            }
        }
    }
}

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

    pub fn assigned(&self, lit: Literal) -> bool {
        self.0[lit.id] == Some(lit.negated)
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (id, negated) in self.0.iter().enumerate() {
            write!(f, "{}{} ", if negated.unwrap() { "-" } else { "" }, id + 1)?;
        }
        write!(f, "0")
    }
}