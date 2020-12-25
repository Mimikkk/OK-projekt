use crate::jssp::*;
use crate::jssp::can::Candidate;
use std::mem::swap;

pub struct Genetic {
    process: BlackBox,
    termination_counter: usize,
    termination_limit: usize,
}

impl Genetic {
    pub fn new(instance: &Instance, termination_limit: usize) -> Self {
        Self {
            process: BlackBox::new(instance),
            termination_counter: 0,
            termination_limit,
        }
    }
    fn find_clear_length(&mut self, p: &mut Vec<Candidate>, mu: usize) -> usize {
        let mut last_makespan = usize::MIN;

        let mut unique_count = 0;
        let mut index = 0;
        while unique_count < mu && index < p.len() {
            if p[index].makespan > last_makespan {
                if index > unique_count { p.swap(unique_count, index); }

                unique_count += 1;
                last_makespan = p[index].makespan;
            }
            index += 1;
        }
        unique_count
    }

    pub fn solve(&mut self, crossover_chance: f64, mu: usize, lambda: usize) -> BlackBox {
        let length = mu + lambda;
        let mut candidates: Vec<Candidate> = (0..length).into_iter().map(|_|
            <BlackBox as NullaryOperator>::apply(&mut self.process)).collect();

        while !self.should_terminate() {
            println!("{}", self.termination_counter);
            candidates.sort_by_key(|x| x.makespan);
            self.process.update_history(&candidates[0]);

            let u = self.find_clear_length(&mut candidates, mu);

            let (a, b) = candidates.split_at_mut(u);
            a.shuffle(&mut self.process.random);
            candidates = [a, b].concat();

            let mut p1 = 0;
            for i in u..length {
                candidates[i] = if self.process.random.gen_bool(crossover_chance) {
                    let mut p2 = self.process.random.gen_range(0, u);
                    while p1 == p2 { p2 = self.process.random.gen_range(0, u) }
                    <BlackBox as BinaryOperator>::apply(&mut self.process, &candidates[p1], &candidates[p2])
                } else { <BlackBox as UnaryOperatorNSwap>::apply(&mut self.process, &candidates[p1]) };
                p1 = (p1 + 1) % u;
            }
        }

        candidates.sort_by_key(|x| x.makespan);
        self.process.update(&candidates[0]);
        self.process.save("genetic").expect("Failed to save.");
        self.process.clone()
    }
}

impl TerminationCriterion for Genetic {
    fn should_terminate(&mut self) -> bool {
        self.termination_counter += 1;
        self.termination_counter >= self.termination_limit
    }
}
