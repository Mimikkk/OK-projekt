#![allow(dead_code)]
#![allow(unused_imports)]

use itertools::{Itertools, zip};
use std::cmp::{max, min};
use std::io::{BufReader, BufRead, Write};
use std::path::Path;
use std::fs::File;
use std::rc::Rc;
use rand::prelude::{ThreadRng, StdRng};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng, SeedableRng};
use crate::jssp::can::Candidate;
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use std::collections::hash_set::Union;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use futures::io::Error;
use std::fmt::Display;
use serde::__private::Formatter;
use custom_error::custom_error;

pub mod can;
pub mod rs;
pub mod hc;
pub mod ga;
pub mod sa;

#[derive(Clone)]
pub enum InstanceType {
    ORLIB,
    TAILLARD,
}

impl Display for InstanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            InstanceType::ORLIB => write!(f, "Orlib"),
            InstanceType::TAILLARD => write!(f, "Taillard"),
        }
    }
}

struct InstanceReader<'a> {
    name: &'a str,
    type_: &'a InstanceType,
}

impl<'a> InstanceReader<'a> {
    fn read(name: &'a str, type_: &'a InstanceType) -> Vec<Vec<usize>> {
        Self { name, type_ }.read_instance_data()
    }

    fn read_instance_data(&self) -> Vec<Vec<usize>> {
        match self.type_ {
            InstanceType::ORLIB => self.read_orlib(),
            InstanceType::TAILLARD => self.read_taillard(),
        }
    }

    fn read_orlib(&self) -> Vec<Vec<usize>> {
        let s = format!("instances\\{}.txt", self.name);
        let path = Path::new(s.as_str());

        BufReader::new(File::open(path).unwrap()).lines().skip(1).map(|x| {
            x.unwrap().split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect()
        }).collect()
    }

    fn read_taillard(&self) -> Vec<Vec<usize>> {
        let s = format!("instances\\{}.txt", self.name);
        let path = Path::new(s.as_str());

        let contents = BufReader::new(File::open(path).unwrap());

        let mut a: usize = 0;
        let mut _b: usize = 0;
        let mut times: Vec<Vec<usize>> = Vec::new();
        let mut machines: Vec<Vec<usize>> = Vec::new();
        for (i, line) in contents.lines().skip(1)
            .map(|x| x.unwrap()).enumerate() {
            if i == 0 {
                let vec: Vec<usize> = line.split_whitespace()
                    .map(|x| x.parse().unwrap()).collect_vec();
                a = vec[0];
                _b = vec[1];
            } else if i == 1 || i == a + 2 { continue; } else if i < a + 2 {
                times.push(line.split_whitespace().map(|x|
                    x.parse::<usize>().unwrap()).collect())
            } else {
                machines.push(line.split_whitespace().map(|x|
                    x.parse::<usize>().unwrap() - 1).collect())
            }
        }

        zip(times, machines).map(|(x, y)|
            y.into_iter().interleave(x).collect()).collect()
    }
}

#[derive(Clone)]
pub struct Instance {
    jobs: Vec<Vec<usize>>,
    name: String,
    type_: InstanceType,
    m: usize,
    n: usize,
    termination_limit: usize,
    is_timed: bool,
}

impl Instance {
    pub fn new(name: &str, type_: InstanceType, termination_limit: usize, is_timed: bool) -> Self {
        let instance_data = InstanceReader::read(name, &type_);
        Self {
            name: String::from(name),
            type_,
            termination_limit,
            is_timed,

            n: instance_data.len(),
            m: instance_data[0].len() / 2,
            jobs: instance_data,
        }
    }
}

impl Serialize for Instance {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where S: Serializer {
        let mut state = serializer.serialize_struct("instance", 7)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("type", &self.type_.to_string())?;
        state.serialize_field("machine_count", &self.m)?;
        state.serialize_field("job_count", &self.n)?;
        state.serialize_field("termination_limit", &self.termination_limit)?;
        state.serialize_field("is_timed", &self.is_timed)?;
        state.serialize_field("data", &self.jobs)?;
        state.end()
    }
}

#[derive(Clone, Serialize)]
pub struct CandidateSchedule { pub schedule: Vec<Vec<usize>> }

impl CandidateSchedule { fn new(m: usize, n: usize) -> Self { Self { schedule: vec![vec![0; 3 * n]; m] } } }

trait RepresentationMapping { fn map(&self, order: &Vec<usize>) -> CandidateSchedule; }

impl RepresentationMapping for BlackBox {
    fn map(&self, order: &Vec<usize>) -> CandidateSchedule {
        let mut machine_state = vec![0; self.instance.m];
        let mut machine_time = vec![0; self.instance.m];
        let mut job_state = vec![0; self.instance.n];
        let mut job_time = vec![0; self.instance.n];

        let jobs = self.instance.jobs.clone();
        let mut y = CandidateSchedule::new(self.instance.m, self.instance.n);

        let (mut machine, mut job_step): (usize, usize);
        let (mut start, mut end): (usize, usize);
        for &job in order.iter() {
            job_step = job_state[job] * 2;
            machine = jobs[job][job_step];
            job_state[job] += 1;

            start = max(machine_time[machine], job_time[job]);
            end = start + jobs[job][job_step + 1];

            machine_time[machine] = end;
            job_time[job] = end;

            y.schedule[machine][machine_state[machine]] = job;
            y.schedule[machine][machine_state[machine] + 1] = start;
            y.schedule[machine][machine_state[machine] + 2] = end;
            machine_state[machine] += 3;
        }
        y
    }
}

trait SearchSpace { fn create(&self) -> Vec<usize>; }

pub struct Counter;

pub struct Time;

pub trait TerminationCriterion<T> { fn should_terminate(&mut self) -> bool; }

pub trait NullaryOperator { fn apply(&mut self) -> Candidate; }

pub trait UnaryOperator1Swap { fn apply(&mut self, based: &Candidate) -> Candidate; }

pub trait UnaryOperatorNSwap { fn apply(&mut self, based: &Candidate) -> Candidate; }

pub trait BinaryOperator { fn apply(&mut self, based_a: &Candidate, based_b: &Candidate) -> Candidate; }

#[derive(Clone)]
pub struct BlackBox {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,

    metaheurestic: String,
    instance: Instance,
    termination_counter: usize,
    timer: std::time::Instant,

    random: StdRng,
    best_candidate: Candidate,
    history: Vec<(f64, usize)>,

    lower_bound: usize,
    upper_bound: usize,
    should_terminate: fn(&mut Self) -> bool,
}

impl Serialize for BlackBox {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        let mut state = serializer.serialize_struct("candidate", 4)?;
        state.serialize_field("author", &"Daniel Zdancewicz");

        #[derive(Serialize)]
        struct InfoData<'a> {
            instance: &'a Instance,
            upper_bound: usize,
            lower_bound: usize,
            metaheurestic: String,
            start: String,
            end: String,
            timetaken: String,
            iteration_count: usize,
        }
        state.serialize_field("info", &InfoData {
            start: self.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            end: self.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            lower_bound: self.lower_bound,
            upper_bound: self.upper_bound,
            timetaken: (self.end_time - self.start_time).to_string(),
            metaheurestic: self.metaheurestic.clone(),
            instance: &self.instance,
            iteration_count: self.termination_counter,
        });
        state.serialize_field("solution", &self.best_candidate);
        state.serialize_field("history", &self.history);
        state.end()
    }
}

impl BlackBox {
    fn new(instance: Instance, metaheurestic: String) -> Self {
        let should_terminate: fn(&mut Self) -> bool = match instance.is_timed {
            true => <Self as TerminationCriterion<Time>>::should_terminate,
            false => <Self as TerminationCriterion<Counter>>::should_terminate,
        };

        let mut bb = Self {
            instance,
            should_terminate,
            metaheurestic,
            start_time: Utc::now(),
            end_time: Utc::now(),
            random: StdRng::from_entropy(),
            best_candidate: Candidate { schedule: vec![], order: vec![], makespan: 0 },
            history: Vec::new(),

            lower_bound: 0,
            upper_bound: 0,

            termination_counter: 0,
            timer: std::time::Instant::now(),
        };

        bb.best_candidate = <Self as NullaryOperator>::apply(&mut bb);
        bb.lower_bound = bb.find_lower_bound();
        bb.upper_bound = bb.find_upper_bound();
        bb
    }

    pub(crate) fn finalize(mut self) -> Self {
        self.end_time = Utc::now();
        self
    }

    fn update(&mut self, candidate: &Candidate) {
        self.update_history(candidate);
        self.update_candidate(candidate);
    }

    fn update_candidate(&mut self, candidate: &Candidate) {
        self.best_candidate = candidate.clone();
    }

    fn update_history(&mut self, candidate: &Candidate) {
        let current_time = self.timer.elapsed().as_secs_f64();

        match self.history.last() {
            None => {
                self.history.push((current_time, candidate.makespan));
            }
            Some(&(prev_time, makespan)) => {
                if makespan > candidate.makespan || current_time - prev_time > 0.01 {
                    self.history.push((current_time, candidate.makespan));
                }
            }
        }
    }

    pub fn save_to_file(&self) -> std::io::Result<()> {
        let data = serde_json::to_string_pretty(&self)?;
        let filepath = format!("solutions/{}.json", self.start_time.format("%d%m%Y-%H-%M-%S"));
        let mut fp = File::create(Path::new(filepath.as_str()))?;
        write!(fp, "{}", data)?;
        Ok(())
    }

    fn find_makespan(&self, y: &CandidateSchedule) -> usize {
        y.schedule.iter().map(|x| *x.last().unwrap())
            .max().expect("Failed to find makespan.")
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
        self.instance.jobs.iter().map(|j| (1..j.len())
            .step_by(2).rev().map(|x| j[x]).sum::<usize>()).sum()
    }
}

impl SearchSpace for BlackBox {
    fn create(&self) -> Vec<usize> {
        (0..self.instance.n).map(|i| vec![i; self.instance.m]).flatten().collect()
    }
}

impl TerminationCriterion<Counter> for BlackBox {
    fn should_terminate(&mut self) -> bool {
        self.termination_counter += 1;
        self.termination_counter >= self.instance.termination_limit
    }
}

impl TerminationCriterion<Time> for BlackBox {
    fn should_terminate(&mut self) -> bool {
        self.termination_counter += 1;
        self.timer.elapsed().as_secs() as usize >= self.instance.termination_limit
    }
}

impl NullaryOperator for BlackBox {
    fn apply(&mut self) -> Candidate {
        let mut vec = self.create();
        vec.shuffle(&mut self.random);
        Candidate::new(&vec, self)
    }
}

impl UnaryOperator1Swap for BlackBox {
    fn apply(&mut self, based: &Candidate) -> Candidate {
        let mut result = based.order.clone();

        let high = based.order.len();
        let (i, mut j) = (self.random.gen_range(0..high), self.random.gen_range(0..high));
        while result[i] == result[j] { j = self.random.gen_range(0..high) }
        result.swap(i, j);
        Candidate::new(&result, self)
    }
}

impl UnaryOperatorNSwap for BlackBox {
    fn apply(&mut self, based: &Candidate) -> Candidate {
        let mut result = based.order.clone();
        let high = based.order.len();

        loop {
            let (i, mut j) = (self.random.gen_range(0..high), self.random.gen_range(0..high));
            while result[i] == result[j] { j = self.random.gen_range(0..high) }
            result.swap(i, j);

            if self.random.gen() { break; }
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
