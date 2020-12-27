use crate::jssp::*;

pub struct RandomSample { process: BlackBox }

impl RandomSample {
    pub fn new(instance: &Instance) -> Self {
        Self {
            process: BlackBox::new(instance),
        }
    }

    pub fn solve(&mut self) -> BlackBox {
        println!("aa");
        let mut solution: Candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
        let mut best_solution = solution.clone();
        while !(self.process.should_terminate)(&mut self.process) {
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
