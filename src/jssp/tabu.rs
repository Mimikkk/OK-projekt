use crate::jssp::*;

pub struct Tabu {
    process: BlackBox,

    tabu_limit: usize,

    termination_counter: usize,
    termination_limit: usize,
}

impl Tabu {
    pub fn new(instance: &Instance, tabu_limit: usize, termination_limit: usize) -> Self {
        Self {
            process: BlackBox::new(instance),

            tabu_limit,
            termination_counter: 0,
            termination_limit,
        }
    }

    pub fn solve(&mut self) -> BlackBox {
        let mut best_solution: Candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);

        let mut taboo_list = vec![best_solution.clone()];
        while !self.should_terminate() {
            let sol: Candidate = (0..self.tabu_limit).map(|_|
                <BlackBox as UnaryOperatorNSwap>::apply(&mut self.process, &best_solution)).max().unwrap();
            if sol > best_solution {
                best_solution = sol;
                self.process.update_history(&best_solution);
            }
        }

        self.process.update(&best_solution);
        self.process.save("tabu").expect("Failed to Save.");
        self.process.clone()
    }
}

impl TerminationCriterion for Tabu {
    fn should_terminate(&mut self) -> bool {
        self.termination_counter += 1;
        self.termination_counter >= self.termination_limit
    }
}