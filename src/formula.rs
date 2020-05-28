use std::ops::Not;
use crate::formula::solver::Assignment;

pub mod parser;
pub mod solver;

pub struct Formula {
    clauses: Vec<(Clause, bool)>,
    assignment: Assignment,
    // reverse index from literals to indices of clauses that have those literals and whether the literal is negated
    clause_indices: Vec<Vec<(usize, bool)>>,
    remaining_clauses: usize,
    unsolvable: bool,
    next_literal_id: usize,
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

    fn unsolvable(&self, formula: &Formula) -> bool {
        self.iter(formula).count() == 0
    }

    fn iter<'a>(&'a self, formula: &'a Formula) -> ClauseIter {
        ClauseIter {
            iter: self.0.iter(),
            assignment: &formula.assignment,
        }
    }
}

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