use crate::jssp::*;
use crate::jssp::can::Candidate;

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
    pub fn solve(&mut self, crossover_chance: f64, mu: usize, lambda: usize) -> BlackBox {
        let length = mu + lambda;
        let mut p: Vec<Candidate> = (0..length).into_iter().map(|_|
            <BlackBox as NullaryOperator>::apply(&mut self.process)).collect_vec();

        while !self.should_terminate() {
            p.sort_by_key(|x| x.makespan);
            self.process.update_history(&p[0]);
            let (a, b) = p.split_at_mut(mu);
            a.shuffle(&mut self.process.random);
            p = [a, b].concat();

            let mut p1 = 0;
            for i in mu..length {
                p[i] = if self.process.random.gen_bool(crossover_chance) {
                    let mut p2 = self.process.random.gen_range(0, mu);
                    while p1 == p2 { p2 = self.process.random.gen_range(0, mu) }
                    <BlackBox as BinaryOperator>::apply(&mut self.process, &p[p1], &p[p2])
                } else {
                    <BlackBox as UnaryOperatorNSwap>::apply(&mut self.process, &p[p1])
                };
                p1 = (p1 + 1) % mu;
            }
        }

        p.sort_by_key(|x| x.makespan);
        self.process.update(&p[0]);
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
