use std::ops::Not;
use std::fmt;

pub mod parser;
pub mod solver;

/// A propositional formula in CNF
pub struct Formula {
    clauses: Vec<(Clause, bool)>,
    assignment: Assignment,
    // reverse index from literals to indices of clauses that have those literals and whether the literal is negated
    clause_indices: Vec<Vec<(usize, bool)>>,
    assign_history: Vec<Vec<Literal>>,
    remaining_clauses: usize,
    unsolvable: bool,
    next_literal_id: usize,
}

impl Formula {
    /// Assign a literal, performing unit propagation
    fn assign(&mut self, lit: Literal) {
        self.next_literal_id = lit.id + 1;
        self.assign_history.push(vec![]);

        fn inner(formula: &mut Formula, lit: Literal) {
            formula.assignment.assign(lit);
            formula.assign_history.last_mut().unwrap().push(lit);
            for i in 0..formula.clause_indices[lit.id].len() {
                let (clause_idx, negated) = formula.clause_indices[lit.id][i];
                if lit.negated != negated {
                    let clause = &formula.clauses[clause_idx].0;
                    let num_literals = clause.num_literals(formula);
                    if num_literals == 0 {
                        formula.unsolvable = true;
                        return;
                    } else if num_literals == 1 {
                        // TODO: Optimise?
                        let unit_lit = clause.iter(formula).next().unwrap();
                        inner(formula, unit_lit);
                    }
                } else if !formula.clauses[clause_idx].1 {
                    formula.clauses[clause_idx].1 = true;
                    formula.remaining_clauses -= 1;
                }
            }
        }

        inner(self, lit);
    }

    /// Un-assign a literal, undoing unit propagation
    fn un_assign(&mut self, lit: Literal) {
        self.next_literal_id = lit.id;
        self.unsolvable = false;
        for lit in self.assign_history.pop().unwrap() {
            self.assignment.un_assign(lit);
            for &(clause_idx, negated) in &self.clause_indices[lit.id] {
                if lit.negated == negated {
                    let (clause, removed) = &self.clauses[clause_idx];
                    if *removed && !clause.solved(self) {
                        self.clauses[clause_idx].1 = false;
                        self.remaining_clauses += 1;
                    }
                }
            }
        }
    }
}

impl<'a> IntoIterator for &'a Formula {
    type Item = &'a Clause;
    type IntoIter = FormulaIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FormulaIter {
            iter: self.clauses.iter(),
        }
    }
}

/// A disjunction of some literals
pub struct Clause(Vec<Literal>);

impl Clause {
    fn new() -> Self {
        Clause(vec![])
    }

    fn add(&mut self, lit: Literal) {
        self.0.push(lit);
    }

    fn solved(&self, formula: &Formula) -> bool {
        self.0.iter().any(|&l| formula.assignment.assigned(l))
    }

    fn num_literals(&self, formula: &Formula) -> usize {
        self.iter(formula).count()
    }

    fn iter<'a>(&'a self, formula: &'a Formula) -> ClauseIter {
        ClauseIter {
            iter: self.0.iter(),
            assignment: &formula.assignment,
        }
    }

    /// If the clauses contains one literal, return it, None otherwise
    fn get_unit_literal(&self) -> Option<Literal> {
        if self.0.len() == 1 {
            Some(unsafe { *self.0.get_unchecked(0) })
        } else {
            None
        }
    }
}

/// A propositional variable (p, q, etc.) with some id which may be negated
/// Ex.: p, !q
#[derive(Copy, Clone, PartialEq)]
pub struct Literal {
    id: usize,
    negated: bool,
}

impl Literal {
    fn from_var(var: isize) -> Self {
        Literal {
            id: var.abs() as usize - 1,
            negated: var < 0,
        }
    }
}

impl Not for Literal {
    type Output = Self;

    fn not(self) -> Self::Output {
        Literal {
            id: self.id,
            negated: !self.negated,
        }
    }
}

pub struct FormulaIter<'a> {
    iter: std::slice::Iter<'a, (Clause, bool)>,
}

impl<'a> Iterator for FormulaIter<'a> {
    type Item = &'a Clause;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((c, d)) =>
                if *d { self.next() } else { Some(c) },
            None => None
        }
    }
}

pub struct ClauseIter<'a> {
    iter: std::slice::Iter<'a, Literal>,
    assignment: &'a Assignment,
}

impl<'a> Iterator for ClauseIter<'a> {
    type Item = Literal;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(&l) =>
                if self.assignment.assigned(!l) { self.next() } else { Some(l) },
            None => None
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