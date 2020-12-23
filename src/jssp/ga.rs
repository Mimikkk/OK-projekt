use crate::jssp::*;
use rand::Rng;
use std::mem::replace;

#[derive(Clone)]
struct Candidate {
    order: Vec<usize>,
    makespan: usize,
}

impl Candidate {
    fn new(mut order: Vec<usize>, process: &mut BlackBox, mut random: &mut ThreadRng) -> Self {
        order.shuffle(&mut random);
        let sol = process.mapping.map(&order);
        let makespan = process.find_makespan(&sol);

        Self { order, makespan }
    }
}

pub struct Genetic {
    process: BlackBox,
    termination_counter: usize,
    termination_limit: usize,
}

impl Genetic {
    pub fn new(instance: &Instance, termination_limit: usize) -> Self {
        Self {
            process: BlackBox::new(instance.clone()),

            termination_counter: 0,
            termination_limit,
        }
    }
    pub fn solve(&mut self, crossover_chance: f64, lambda: usize, mu: usize) -> BlackBox {
        let mut random = &mut thread_rng();
        let length = lambda + mu;
        let mut p = (0..length).into_iter().map(|_|
            Candidate::new(<Self as NullaryOperator>::apply(&self, &mut random),
                           &mut self.process,
                           &mut random)).collect_vec();

        while !self.should_terminate() {
            p.sort_by_key(|x| x.makespan);
            // self.process.update(&p[0].order);
            let (a, b) = p.split_at_mut(mu);
            let (mut p1, mut p2): (usize, usize) = (0, random.gen_range(0, mu));
            a.shuffle(&mut random);
            p = [a, b].concat();

            for i in mu..length {
                let new_order =
                    if random.gen_bool(crossover_chance) {
                        while p1 == p2 { p2 = random.gen_range(0, mu) }
                        <Self as BinaryOperator>::apply(&self, &p[p1].order, &p[p2].order, &mut random)
                    } else {
                        <Self as UnaryOperatorNSwap>::apply(&self, &p[p1].order, &mut random)
                    };
                p[i] = Candidate::new(new_order, &mut self.process, &mut random);
                p1 = (p1 + 1) % mu;
            }
        }

        p.sort_by_key(|x| x.makespan);
        self.process.update(&p[0].order);
        let name = format!("genetic");
        self.process.save(name.as_str()).expect("Failed to save.");
        self.process.clone()
    }
}

impl TerminationCriterion for Genetic {
    fn should_terminate(&mut self) -> bool {
         println!("{}", self.termination_counter);
        self.termination_counter += 1;
        self.termination_counter >= self.termination_limit
    }
}

impl NullaryOperator for Genetic {
    fn apply(&self, random: &mut ThreadRng) -> Vec<usize> {
        let mut vec = self.process.search_space.create();
        vec.shuffle(random);
        vec
    }
}

impl UnaryOperator1Swap for Genetic {}

impl UnaryOperatorNSwap for Genetic {}

impl BinaryOperator for Genetic {}