use crate::jssp::*;

pub struct RandomSample {
    process: BlackBox,

    termination_counter: usize,
    termination_limit: usize,
}

impl RandomSample {
    pub fn new(instance: &Instance, termination_limit: usize) -> Self {
        Self {
            process: BlackBox::new(instance),
            termination_counter: 0,
            termination_limit,
        }
    }

    pub fn solve(&mut self) -> BlackBox {
        let mut solution: Candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
        let mut best_solution = solution.clone();

        while !self.should_terminate() {
            solution = <BlackBox as NullaryOperator>::apply(&mut self.process);

            if solution > best_solution {
                best_solution = solution;
                self.process.update_history(&best_solution);
            }
        }

        self.process.update(&best_solution);
        self.process.save("random_sample").expect("Failed to Save.");
        self.process.clone()
    }
}

impl TerminationCriterion for RandomSample {
    fn should_terminate(&mut self) -> bool {
        self.termination_counter += 1;
        return self.termination_counter >= self.termination_limit;
    }
}
