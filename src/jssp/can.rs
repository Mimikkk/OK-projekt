use crate::jssp::{BlackBox, RepresentationMapping, CandidateSchedule};
use std::cmp::{Ordering};
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

#[derive(Clone, Serialize)]
pub struct Candidate {
    pub makespan: usize,
    pub order: Vec<usize>,
    pub schedule: Vec<Vec<usize>>,
}

impl Candidate {
    pub fn new(order: &Vec<usize>, process: &mut BlackBox) -> Self {
        let sol = process.map(&order);
        Self { order: order.clone(), makespan: process.find_makespan(&sol), schedule: sol.schedule }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.makespan.partial_cmp(&self.makespan)
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.makespan.cmp(&other.makespan)
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl Eq for Candidate {}