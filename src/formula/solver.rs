use super::*;

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
}