#![allow(dead_code)]
#![allow(unused_imports)]

use itertools::{Itertools, zip};
use std::cmp::{max, min};
use std::io::{BufReader, BufRead, Write};
use std::path::Path;
use std::fs::File;
use std::rc::Rc;
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use crate::jssp::can::Candidate;

pub mod rs;
pub mod hc;
pub mod ga;
pub mod can;
pub mod tabu;

pub trait TerminationCriterion {
    fn should_terminate(&mut self) -> bool;
}

pub trait NullaryOperator {
    fn apply(&mut self) -> Candidate;
}

pub trait UnaryOperator1Swap {
    fn apply(&mut self, based: &Candidate) -> Candidate;
}

pub trait UnaryOperatorNSwap {
    fn apply(&mut self, based: &Candidate) -> Candidate;
}

pub trait BinaryOperator {
    fn apply(&mut self, based_a: &Candidate, based_b: &Candidate) -> Candidate;
}

impl NullaryOperator for BlackBox {
    fn apply(&mut self) -> Candidate {
        let mut vec = self.search_space.create();
        vec.shuffle(&mut self.random);
        Candidate::new(&vec, self)
    }
}

impl UnaryOperator1Swap for BlackBox {
    fn apply(&mut self, based: &Candidate) -> Candidate {
        let mut result = based.order.clone();

        let high = based.order.len();
        let (i, mut j) = (self.random.gen_range(0, high), self.random.gen_range(0, high));
        while result[i] == result[j] { j = self.random.gen_range(0, high) }
        result.swap(i, j);
        Candidate::new(&result, self)
    }
}

impl UnaryOperatorNSwap for BlackBox {
    fn apply(&mut self, based: &Candidate) -> Candidate {
        let mut result = based.order.clone();
        let high = based.order.len();

        let mut should_flip = true;
        while should_flip {
            let (i, mut j) = (self.random.gen_range(0, high), self.random.gen_range(0, high));
            while result[i] == result[j] { j = self.random.gen_range(0, high) }
            result.swap(i, j);

            should_flip = self.random.gen();
        }
        Candidate::new(&result, self)
    }
}

impl BinaryOperator for BlackBox {
    fn apply(&mut self, based_a: &Candidate, based_b: &Candidate) -> Candidate {
        let length: usize = based_a.order.len();

        let mut visited_a: Vec<bool> = vec![false; length];
        let mut visited_b: Vec<bool> = vec![false; length];

        let mut result: Vec<usize> = vec![0; length];
        let mut result_i: usize = 0;

        let mut a_i: usize = 0;
        let mut b_i: usize = 0;

        loop {
            let add = if self.random.gen::<bool>() { based_a.order[a_i] } else { based_b.order[b_i] };
            result[result_i] = add;
            result_i += 1;

            if result_i >= length { return Candidate::new(&result, self); }

            let mut i = a_i;
            loop {
                if based_a.order[i] == add && !visited_a[i] {
                    visited_a[i] = true;
                    break;
                };
                i += 1
            }
            while visited_a[a_i] { a_i += 1 }

            let mut i = b_i;
            loop {
                if based_b.order[i] == add && !visited_b[i] {
                    visited_b[i] = true;
                    break;
                };
                i += 1
            }
            while visited_b[b_i] { b_i += 1 }
        }
    }
}

#[derive(Clone)]
pub struct Instance {
    jobs: Rc<Vec<Vec<usize>>>,
    id: String,
    m: usize,
    n: usize,
}

impl Instance {
    pub fn new(instance_name: &str, instance_type: &str) -> Self {
        let instance_data: Vec<Vec<usize>> = match instance_type.to_lowercase().as_str() {
            "orlib" => { Self::read_orlib(instance_name.to_string()) }
            "taillard" => { Self::read_taillard(instance_name.to_string()) }
            _ => { panic!("Wrong instance type {}", instance_type) }
        };
        if instance_data.is_empty() { panic!("Empty instance") }

        Self {
            n: instance_data.len(),
            m: instance_data[0].len() / 2,
            jobs: Rc::new(instance_data),
            id: instance_name.to_string(),
        }
    }

    fn read_orlib(instance_name: String) -> Vec<Vec<usize>> {
        let s = format!("instances\\{}.txt", instance_name);
        let path = Path::new(s.as_str());

        let contents = BufReader::new(File::open(path)
            .expect("failed to load the file"));

        contents.lines().skip(1).map(|x| {
            x.expect("Failed to map a line")
                .split_whitespace()
                .map(|x| x.parse()
                    .expect("Failed to parse the number."))
                .collect()
        }).collect()
    }

    fn read_taillard(instance_name: String) -> Vec<Vec<usize>> {
        let s = format!("instances\\{}.txt", instance_name);
        let path = Path::new(s.as_str());

        let contents = BufReader::new(File::open(path)
            .expect("failed to load the file"));

        let mut a: usize = 0;
        let mut _b: usize = 0;
        let mut times: Vec<Vec<usize>> = Vec::new();
        let mut machines: Vec<Vec<usize>> = Vec::new();
        for (i, line) in contents.lines().skip(1)
            .map(|x| x.expect("line wasn't read properly")).enumerate() {
            if i == 0 {
                let vec: Vec<usize> = line.split_whitespace()
                    .map(|x| x.parse().expect("Not parse the number.")).collect_vec();
                a = vec[0];
                _b = vec[1];
            } else if i == 1 || i == a + 2 { continue; } else if i < a + 2 {
                times.push(line.split_whitespace().map(|x|
                    x.parse::<usize>().expect("Invalid digit.")).collect())
            } else {
                machines.push(line.split_whitespace().map(|x|
                    x.parse::<usize>().expect("Invalid digit.") - 1).collect())
            }
        }

        zip(times, machines).map(|(x, y)|
            y.into_iter().interleave(x).collect()).collect()
    }
}

#[derive(Clone)]
pub struct CandidateMapping {
    pub schedule: Vec<Vec<usize>>,
}

impl CandidateMapping {
    fn new(m: usize, n: usize) -> Self {
        Self { schedule: vec![vec![0; 3 * n]; m] }
    }
}

#[derive(Clone)]
struct SearchSpace {
    instance: Rc<Instance>,
}

impl SearchSpace {
    fn new(instance: Rc<Instance>) -> Self {
        Self { instance: Rc::clone(&instance) }
    }

    fn create(&self) -> Vec<usize> {
        (0..self.instance.n).map(|i| vec![i; self.instance.m]).flatten().collect()
    }
}

#[derive(Clone)]
struct RepresentationMapping {
    job_time: Vec<usize>,
    job_state: Vec<usize>,
    machine_time: Vec<usize>,
    machine_state: Vec<usize>,
    jobs: Rc<Vec<Vec<usize>>>,
}

impl RepresentationMapping {
    fn new(instance: Rc<Instance>) -> Self {
        Self {
            job_time: vec![0; instance.n],
            job_state: vec![0; instance.n],
            machine_time: vec![0; instance.m],
            machine_state: vec![0; instance.m],
            jobs: Rc::clone(&instance.jobs),
        }
    }

    fn map(&mut self, x: &Vec<usize>) -> CandidateMapping {
        self.machine_state.iter_mut().for_each(|x| *x = 0);
        self.machine_time.iter_mut().for_each(|x| *x = 0);
        self.job_time.iter_mut().for_each(|x| *x = 0);
        self.job_state.iter_mut().for_each(|x| *x = 0);
        let mut y =
            CandidateMapping::new(self.machine_time.len(), self.job_time.len());

        let mut machine: usize;
        let mut job_step: usize;

        let mut start: usize;
        let mut end: usize;

        for &job in x.iter() {
            job_step = self.job_state[job] * 2;
            machine = self.jobs[job][job_step];
            self.job_state[job] += 1;

            start = max(self.machine_time[machine], self.job_time[job]);
            end = start + self.jobs[job][job_step + 1];

            self.machine_time[machine] = end;
            self.job_time[job] = end;

            y.schedule[machine][self.machine_state[machine]] = job;
            y.schedule[machine][self.machine_state[machine] + 1] = start;
            y.schedule[machine][self.machine_state[machine] + 2] = end;
            self.machine_state[machine] += 3;
        }
        y
    }
}

#[derive(Clone)]
pub struct BlackBox {
    instance: Rc<Instance>,
    search_space: SearchSpace,
    mapping: RepresentationMapping,
    random: ThreadRng,

    pub best_candidate: Candidate,
    pub history: Vec<usize>,

    pub lower_bound: usize,
    pub upper_bound: usize,
}

impl BlackBox {
    fn new(instance: &Instance) -> Self {
        let p_instance = Rc::new(instance.clone());

        let mut bb = Self {
            search_space: SearchSpace::new(Rc::clone(&p_instance)),
            mapping: RepresentationMapping::new(Rc::clone(&p_instance)),
            instance: p_instance,

            random: thread_rng(),
            best_candidate: Candidate { order: vec![], makespan: 0 },
            history: Vec::new(),

            lower_bound: 0,
            upper_bound: 0,
        };

        bb.best_candidate = <Self as NullaryOperator>::apply(&mut bb);
        bb.lower_bound = bb.find_lower_bound();
        bb.upper_bound = bb.find_upper_bound();
        bb
    }
    fn update(&mut self, candidate: &Candidate) {
        self.best_candidate = candidate.clone();
        self.update_history(candidate);
    }

    fn update_history(&mut self, candidate: &Candidate) {
        match self.history.last() {
            None => { self.history.push(candidate.makespan); }
            Some(&makespan) => {
                if makespan > candidate.makespan {
                    self.history.push(candidate.makespan);
                }
            }
        }
    }

    fn save(&mut self, name: &str) -> std::io::Result<()> {
        self.save_schedule(name)?;
        self.save_history(name)?;
        Ok(())
    }

    fn save_history(&self, name: &str) -> std::io::Result<()> {
        let s = format!("solutions\\{}_{}_history.txt", self.instance.id, name);
        let path = Path::new(s.as_str());
        let mut file = File::create(path)?;

        for i in self.history.iter() { write!(file, "{} ", i)? }
        write!(file, "\n")?;
        Ok(())
    }

    fn save_schedule(&mut self, name: &str) -> std::io::Result<()> {
        let s = format!("solutions\\{}_{}_solution.txt", self.instance.id, name);
        let path = Path::new(s.as_str());
        let mut file = File::create(path)?;

        writeln!(file, "{} {}", self.instance.n, self.instance.m)?;
        writeln!(file, "{}", self.best_candidate.makespan)?;
        for line in self.mapping.map(&self.best_candidate.order).schedule.iter() {
            for i in line { write!(file, "{} ", i)? }
            write!(file, "\n")?;
        }

        Ok(())
    }

    fn find_makespan(&self, y: &CandidateMapping) -> usize {
        y.schedule.iter().map(|x| *x.last().unwrap()).max().expect("Failed to find makespan.")
    }

    fn find_lower_bound(&mut self) -> usize {
        let mut a: Vec<usize> = vec![usize::MAX; self.instance.m];
        let mut b: Vec<usize> = vec![usize::MAX; self.instance.m];
        let mut t: Vec<usize> = vec![0; self.instance.m];

        let mut bound: usize = 0;
        for i in (0..self.instance.n).rev() {
            let job = &self.instance.jobs[i];

            let job_total_time: usize = (1..job.len()).step_by(2).map(|x| job[x]).sum();
            bound = max(bound, job_total_time);

            let mut time: usize;
            let mut machine: usize;
            let mut job_current_time: usize = 0;
            for i in (0..job.len()).step_by(2) {
                machine = job[i];
                time = job[i + 1];

                a[machine] = min(a[machine], job_current_time);

                t[machine] += time;
                job_current_time += time;
                b[machine] = min(b[machine], job_total_time - job_current_time);
            }
        }

        (0..self.instance.m).map(|i| max(bound, a[i] + b[i] + t[i])).max()
            .expect("Failed to find the final bound")
    }

    fn find_upper_bound(&self) -> usize {
        //Sloppy.
        self.instance.jobs.iter().map(|j| (1..j.len()).step_by(2).rev().map(|x| j[x]).sum::<usize>()).sum()
    }
}
