use crate::jssp::*;
use rand::Rng;

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
    pub fn solve(&mut self, lambda: usize, mi: usize) -> BlackBox {
        let mut random = &mut thread_rng();

        let mut p: Vec<Candidate> =
            vec![Candidate::new(
                <Self as NullaryOperator>
                ::apply(&self, random), &mut self.process, random); lambda+mi];

        let mut p_index: i32;
        while !self.should_terminate(){
            self.termination_counter+=1;
            p.sort_by_key(|x| x.makespan);
            if self.process.history.is_empty()
                || self.process.history.last().unwrap() > &p[0].makespan {
                self.process.update_history(p[0].makespan);
            }

            let (b,c) = p.split_at_mut(mi);
            b.shuffle(random);
            p = [b,c].concat();

            p_index = -1;
            for i in mi..mi+lambda {
                p_index = (p_index+1) % (mi as i32);
                p[i] =
                    Candidate::new(
                        <Self as BinaryOperator>::apply(&self, &p[i].order, &p[p_index as usize].order, random),
                        &mut self.process, random);
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
    fn should_terminate(&self) -> bool {
        return self.termination_counter >= self.termination_limit
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

impl BinaryOperator for Genetic {
    fn apply(&self, based_a: &Vec<usize>, based_b: &Vec<usize>, random: &mut ThreadRng) -> Vec<usize> {
        let length: usize = based_a.len();
        let mut visited_a: Vec<bool> = vec![false; length];
        let mut visited_b: Vec<bool> = vec![false; length];
        let mut result:  Vec<usize> = vec![0; length];

        let mut result_i: usize = 0;
        let mut a_i: usize = 0;
        let mut b_i: usize = 0;
        let mut add: usize = 0;
        loop {
            add = if random.gen::<bool>() {based_a[a_i]} else {based_b[b_i]};
            result[result_i] = add;
            result_i += 1;

            if result_i >= length { return result }

            let mut i = a_i;
            loop { if based_a[i] == add && !visited_a[i] { visited_a[i] = true; break } ; i += 1 }
            while visited_a[a_i] { a_i += 1 }

            let mut i = b_i;
            loop { if based_b[i] == add && !visited_b[i] { visited_b[i] = true; break } ; i += 1 }
            while visited_b[b_i] { b_i += 1 }
        }
    }
}