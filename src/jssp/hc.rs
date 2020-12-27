use crate::jssp::*;

pub struct HillClimber {
    process: BlackBox,

    reset_threshold: usize,
    reset_counter: usize,
}

impl HillClimber {
    pub fn new(instance: &Instance, reset_threshold: usize) -> Self {
        Self {
            process: BlackBox::new(instance),

            reset_threshold,
            reset_counter: 0,
        }
    }

    pub fn solve(&mut self, unary_op: &str) -> BlackBox {
        let mut best_candidate: Candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
        let mut next_candidate;
        let mut prev_candidate: Candidate = best_candidate.clone();

        let search_operator: fn(&mut BlackBox, &Candidate) -> Candidate
            = match unary_op.to_lowercase().as_str() {
            "1swap" => <BlackBox as UnaryOperator1Swap>::apply,
            "nswap" => <BlackBox as UnaryOperatorNSwap>::apply,
            _ => panic!("Unsupported operator"),
        };

        while !<BlackBox as TerminationCriterion<Counter>>::should_terminate(&mut self.process) {
            if self.should_reset() {
                prev_candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
                self.reset_counter = 0;
            }
            next_candidate = search_operator(&mut self.process, &prev_candidate);

            if next_candidate > prev_candidate {
                prev_candidate = next_candidate;
            }

            if prev_candidate > best_candidate {
                best_candidate = prev_candidate.clone();
                self.process.update_history(&best_candidate);
                self.reset_counter = 0;
            }
        }

        self.process.update(&best_candidate);
        let name = format!("hillclimber_{}_restarts", unary_op.to_lowercase());
        self.process.save(name.as_str()).expect("Failed to save.");
        self.process.clone()
    }

    fn should_reset(&mut self) -> bool {
        self.reset_counter += 1;
        self.reset_counter >= self.reset_threshold
    }
}
