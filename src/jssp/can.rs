use crate::jssp::BlackBox;
use std::cmp::{Ordering};

#[derive(Clone)]
pub struct Candidate {
    pub order: Vec<usize>,
    pub makespan: usize,
}

impl Candidate {
    pub fn new(mut order: &Vec<usize>, process: &mut BlackBox) -> Self {
        let sol = process.mapping.map(&order);
        Self { order: order.clone(), makespan: process.find_makespan(&sol) }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.makespan.partial_cmp(&self.makespan)
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl Eq for Candidate {}