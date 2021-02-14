use crate::jssp::*;
use std::thread;


pub struct HillClimber {
    process: BlackBox,
    unary_op: String,
    reset_threshold: usize,
    reset_counter: usize,
}

impl HillClimber {
    pub fn new(instance: &Instance, reset_threshold: usize, unary_op: &str) -> Self {
        Self {
            process: BlackBox::new(instance.clone(), String::from("HillClimber with resets")),
            unary_op: String::from(unary_op),
            reset_threshold,
            reset_counter: 0,
        }
    }

    pub fn solve(&mut self) -> BlackBox {
        let mut best_candidate: Candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
        let mut next_candidate;
        let mut prev_candidate: Candidate = best_candidate.clone();

        let search_operator: fn(&mut BlackBox, &Candidate) -> Candidate
            = match self.unary_op.to_lowercase().as_str() {
            "1swap" => <BlackBox as UnaryOperator1Swap>::apply,
            "nswap" => <BlackBox as UnaryOperatorNSwap>::apply,
            _ => panic!("Unsupported operator"),
        };

        while !(self.process.should_terminate)(&mut self.process) {
            if self.should_reset() {
                prev_candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
                self.reset_counter = 0;
            }
            next_candidate = search_operator(&mut self.process, &prev_candidate);

            if next_candidate > prev_candidate {
                prev_candidate = next_candidate;
            } else {}

            if prev_candidate > best_candidate {
                best_candidate = prev_candidate.clone();
                self.process.update_history(&best_candidate);
                self.reset_counter = 0;
            }
        }
        self.process.update(&best_candidate);
        self.process.clone().finalize()
    }

    pub fn solve_threaded(&self) -> BlackBox {
        let handles = (0..thread::available_concurrency().expect("Failed to get thread count").get())
            .map(|_| {
                let mut hc = Self::new(&self.process.instance, self.reset_threshold, self.unary_op.clone().as_str());
                thread::spawn(move || hc.solve())
            })
            .collect_vec();

        let bbs = handles
            .into_iter()
            .map(|x| x.join().expect("Failed to extract the Black box"))
            .collect_vec();

        println!("Used {} threads", bbs.len());
        println!("With {} total iterations", bbs.iter().fold(0, |a, b| a + b.termination_counter));
        println!("In the time of {}s", self.process.instance.termination_limit);
        bbs.into_iter().max_by_key(|x| x.best_candidate.makespan).unwrap().clone()
    }

    fn should_reset(&mut self) -> bool {
        self.reset_counter += 1;
        self.reset_counter >= self.reset_threshold
    }
}
