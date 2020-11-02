use crate::jssp::*;
use rand::Rng;

pub struct SingleHillClimber {
    process: BlackBox,

    termination_counter: usize,
    termination_limit: usize,

    reset_threshold: usize,
    reset_counter: usize,
}

impl SingleHillClimber {
    pub fn new(instance: &Instance, termination_limit: usize, reset_threshold: usize) -> Self {
        Self {
            process: BlackBox::new(instance.clone()),
            termination_counter: 0,
            termination_limit,

            reset_threshold,
            reset_counter: 0
        }
    }

    pub fn solve(&mut self) -> BlackBox {
        let mut random: ThreadRng = thread_rng();
        let mut best_order = self.process.search_space.create();
        let mut prev_order = best_order.clone();
        let mut next_order = best_order.clone();
        let mut best_makespan = usize::MAX;
        let mut next_makespan = usize::MAX;
        let mut prev_makespan = usize::MAX;
        let mut solution: CandidateSolution;

        while !self.should_terminate() {
            if self.reset_counter >= self.reset_threshold {
                prev_order = self.null_apply(&mut random);
                solution = self.process.mapping.map(&prev_order);
                prev_makespan = self.process.find_makespan(&solution);
                self.reset_counter = 0;
            }

            next_order = self.uno_apply(&prev_order, &mut random);
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
        self.process.save("hill climber-1swaps resets").expect("Failed to save.");
        self.process.clone()
    }
}

impl TerminationCriterion for SingleHillClimber {
    fn should_terminate(&self) -> bool {
        return self.termination_counter >= self.termination_limit
    }
}

impl NullaryOperator for SingleHillClimber {
    fn null_apply(&self, random: &mut ThreadRng) -> Vec<usize> {
        let mut vec = self.process.search_space.create();
        vec.shuffle(random);
        vec
    }
}

impl UnaryOperator1Swap for SingleHillClimber {
    fn uno_apply(&self, based: &Vec<usize>, random: &mut ThreadRng) -> Vec<usize> {
        let mut result = based.clone();
        let high = based.len();
        let mut i: usize = random.gen_range(0, high);
        let mut j: usize = random.gen_range(0, high);
        while result[i] == result[j] { j = random.gen_range(0, high) }
        result.swap(i,j);
        result
    }
}