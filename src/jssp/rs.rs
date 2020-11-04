use crate::jssp::*;

pub struct RandomSample {
    process: BlackBox,

    termination_counter: usize,
    termination_limit: usize,
}

impl RandomSample {
    pub fn new(instance: &Instance, termination_limit: usize) -> Self {
        Self {
            process: BlackBox::new(instance.clone()),
            termination_counter: 0,
            termination_limit,
        }
    }

    pub fn solve(&mut self) -> BlackBox {
        let mut random: ThreadRng = thread_rng();
        let mut order: Vec<usize>;
        let mut makespan: usize;

        let mut best_order = self.apply(&mut random);
        let mut solution: CandidateSolution = self.process.mapping.map(&best_order);
        let mut best_makespan: usize = self.process.find_makespan(
            &solution
        );

        while !self.should_terminate() {
            self.termination_counter += 1;


            order = self.apply(&mut random);
            solution = self.process.mapping.map(&order);
            makespan = self.process.find_makespan(&solution);

            if best_makespan > makespan {
                best_order = order.clone();
                best_makespan = makespan;

                self.process.update_history(best_makespan);
                self.termination_counter = 0;
            }
        }

        self.process.update(&best_order);
        self.process.save("random_sample").expect("Failed to Save.");
        self.process.clone()
    }
}

impl TerminationCriterion for RandomSample {
    fn should_terminate(&self) -> bool {
        return self.termination_counter >= self.termination_limit
    }
}

impl NullaryOperator for RandomSample {
    fn apply(&self, random: &mut ThreadRng) -> Vec<usize> {
        let mut vec = self.process.search_space.create();
        vec.shuffle(random);
        vec
    }
}
