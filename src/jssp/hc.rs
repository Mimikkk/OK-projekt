use crate::jssp::*;

pub struct HillClimber {
    process: BlackBox,

    termination_counter: usize,
    termination_limit: usize,

    reset_threshold: usize,
    reset_counter: usize,
}

impl HillClimber {
    pub fn new(instance: &Instance, termination_limit: usize, reset_threshold: usize) -> Self {
        Self {
            process: BlackBox::new(instance.clone()),
            termination_counter: 0,
            termination_limit,

            reset_threshold,
            reset_counter: 0
        }
    }

    pub fn solve(&mut self, unary_op: &str) -> BlackBox {
        let mut random: ThreadRng = thread_rng();
        let mut best_order = self.process.search_space.create();
        let mut prev_order = best_order.clone();
        let mut next_order: Vec<usize>;
        let mut best_makespan = usize::MAX;
        let mut next_makespan : usize;
        let mut prev_makespan = usize::MAX;
        let mut solution: CandidateSolution;

        let search_operator: fn(&Self, &Vec<usize>, &mut ThreadRng) -> Vec<usize>
            = match unary_op.to_lowercase().as_str() {
            "1swap" => <Self as UnaryOperator1Swap>::apply,
            "nswap" => <Self as UnaryOperatorNSwap>::apply,
            _ => panic!("Unsupported operator"),
        };

        while !self.should_terminate() {
            if self.reset_counter >= self.reset_threshold {
                prev_order = <Self as NullaryOperator>::apply(&self, &mut random);
                solution = self.process.mapping.map(&prev_order);
                prev_makespan = self.process.find_makespan(&solution);
                self.reset_counter = 0;
            }
            next_order = search_operator(&self, &prev_order, &mut random);
            solution = self.process.mapping.map(&next_order);
            next_makespan = self.process.find_makespan(&solution);

            if next_makespan < prev_makespan {
                prev_makespan = next_makespan;
                prev_order = next_order.clone();
            }

            if next_makespan < best_makespan {
                best_makespan = next_makespan;
                best_order = next_order.clone();
                self.process.update_history(best_makespan);
                self.reset_counter = 0;
                self.termination_counter = 0;
            } else { self.reset_counter += 1; }

            self.termination_counter += 1;
        }

        self.process.update(&best_order);
        let name = format!("hillclimber_{}_restarts", unary_op.to_lowercase());
        self.process.save(name.as_str()).expect("Failed to save.");
        self.process.clone()
    }
}

impl TerminationCriterion for HillClimber {
    fn should_terminate(&mut self) -> bool {
        return self.termination_counter >= self.termination_limit
    }
}

impl NullaryOperator for HillClimber {
    fn apply(&self, random: &mut ThreadRng) -> Vec<usize> {
        let mut vec = self.process.search_space.create();
        vec.shuffle(random);
        vec
    }
}

impl UnaryOperator1Swap for HillClimber {}

impl UnaryOperatorNSwap for HillClimber {}
