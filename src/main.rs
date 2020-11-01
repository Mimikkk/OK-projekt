use std::rc::Rc;
use rand::prelude::ThreadRng;
use std::io::{BufReader, BufRead, Write};
use std::fs::File;
use itertools::{Itertools, zip};
use std::cmp::{max, min};
use rand::seq::SliceRandom;
use std::borrow::{BorrowMut, Borrow};
use std::ops::Deref;
use rand::{random, thread_rng};
use std::path::Path;

trait TerminationCriterion {
    fn should_terminate(&self) -> bool;
}

struct BlackBox {
    instance: Rc<Instance>,
    search_space: SearchSpace,
    makespan: MakespanFunction,
    mapping: RepresentationMapping,

    lower_bound: usize,
    upper_bound: usize,
}

impl BlackBox {
    pub fn new(instance: Instance) -> Self{
        let p_instance = Rc::new(instance);
        let mut a = Self {
            search_space: SearchSpace::new(Rc::clone(&p_instance)),
            makespan: MakespanFunction::new(Rc::clone(&p_instance)),
            mapping: RepresentationMapping::new(Rc::clone(&p_instance)),

            lower_bound: 0,
            upper_bound: 0,
            instance: p_instance,
        };
        a.lower_bound = a.makespan.lower_bound();
        a.upper_bound = a.makespan.upper_bound();
        a
    }

    pub fn save_history(&self, history: &Vec<usize>, name: &str) -> std::io::Result<()> {
        let s = format!("solutions\\{}_{}_history.txt", self.instance.id, name);
        let path = Path::new(s.as_str());
        let mut file = File::create(path)?;

        for i in history.iter() { write!(file, "{} ", i)? }
        write!(file, "\n")?;
        Ok(())
    }

    pub fn save_schedule(&self, schedule: &CandidateSolution, name: &str) -> std::io::Result<()> {
        let s = format!("solutions\\{}_{}_solution.txt", self.instance.id, name);
        let path = Path::new(s.as_str());
        let mut file = File::create(path)?;

        writeln!(file, "{}", name)?;
        for line in schedule.schedule.iter() {
            for i in line{ write!(file, "{} ", i)? }
            write!(file, "\n")?;
        }

        Ok(())
    }

}

struct SearchSpace{
    instance: Rc<Instance>,
    m_length: usize,
}

impl SearchSpace {
    fn new(instance: Rc<Instance>) -> Self {
        Self {instance: Rc::clone(&instance), m_length: instance.m}
    }

    fn create(&self) -> Vec<usize> {
        (0..self.instance.n).map(|i| vec![i;self.instance.m]).flatten().collect()
    }

    fn copy(from: Vec<usize>, mut to: Vec<usize>) {
        to = from.clone();
    }
}

struct Instance {
    jobs: Rc<Vec<Vec<usize>>>,
    id: String,
    m: usize,
    n: usize,
}

impl Instance {
    pub fn new(instance_name: &str, instance_type: &str) -> Self{
        let instance_data: Vec<Vec<usize>> = match instance_type.to_lowercase().as_str() {
            "orlib" => { Self::read_orlib(instance_name.to_string()) },
            "taillard" => { Self::read_taillard(instance_name.to_string()) },
            _ => { panic!("Wrong instance type {}", instance_type) },
        };
        if instance_data.is_empty() {panic!("Empty instance")}

        Self {
            n: instance_data.len() ,
            m: instance_data[0].len() /2,
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

        let mut a: usize=0;
        let mut b: usize=0;
        let mut times: Vec<Vec<usize>> = Vec::new();
        let mut machines: Vec<Vec<usize>> = Vec::new();
        for (i, line) in contents.lines().skip(1)
            .map(|x| x.expect("line wasn't read properly")).enumerate() {
            if i == 0 {
                let vec: Vec<usize> = line.split_whitespace()
                    .map(|x| x.parse().expect("Not parse the number.")).collect_vec();
                a = vec[0];
                b = vec[1];
            }
            else if i == 1 || i == a + 2 { continue; }
            else if i < a + 2 {times.push(line.split_whitespace().map(|x|
                x.parse::<usize>().expect("Invalid digit.")).collect())}
            else {machines.push(line.split_whitespace().map(|x|
                x.parse::<usize>().expect("Invalid digit.") - 1).collect())}
        }

        zip(times, machines).map(|(x, y)|
            y.into_iter().interleave(x).collect()).collect()
    }
}

struct CandidateSolution {
    schedule: Vec<Vec<usize>>,
}

impl CandidateSolution {
    pub fn new(m: usize, n: usize) -> Self{
        Self{schedule: vec![vec![0 ; 3*n] ; m]}
    }
}

struct MakespanFunction {
    instance: Rc<Instance>,
}

impl MakespanFunction {
    pub fn new(instance: Rc<Instance>) -> Self{
        Self{instance}
    }

    fn find_makespan(&self, y: &CandidateSolution) -> usize {
        y.schedule.iter().map(|x| *x.last().unwrap()).max().expect("Failed to find makespan.")
    }

    fn lower_bound(&mut self) -> usize {
        let mut a: Vec<usize> = vec![usize::MAX; self.instance.m ];
        let mut b: Vec<usize> = vec![usize::MAX; self.instance.m ];
        let mut t: Vec<usize> = vec![0; self.instance.m ];

        let mut bound: usize = 0;
        for i in (0..self.instance.n ).rev(){
            let job = &self.instance.jobs[i];

            let job_total_time: usize = (1..job.len()).step_by(2).map(|x| job[x]).sum();
            bound = max(bound, job_total_time);

            let mut time: usize;
            let mut machine: usize;
            let mut job_current_time: usize = 0;
            for i in (0..job.len()).step_by(2) {
                machine = job[i] ;
                time = job[i+1];

                a[machine] = min(a[machine], job_current_time);

                t[machine] += time;
                job_current_time += time;
                b[machine] = min(b[machine], job_total_time - job_current_time);
            }
        }

        (0..self.instance.m ).map(|i| max(bound, a[i]+b[i]+t[i])).max()
            .expect("Failed to find the final bound")
    }

    fn upper_bound(&self) -> usize{
        //Sloppy.
        self.instance.jobs.iter().map(|j| (1..j.len()).step_by(2).rev().map(|x| j[x]).sum::<usize>()).sum()
    }
}

struct RepresentationMapping {
    job_time: Vec<usize>,
    job_state: Vec<usize>,
    machine_time: Vec<usize>,
    machine_state: Vec<usize>,
    jobs: Rc<Vec<Vec<usize>>>,
}

impl RepresentationMapping{
    pub fn new(instance: Rc<Instance>) -> Self {
        Self {
            job_time: vec![0;instance.n ],
            job_state: vec![0;instance.n ],
            machine_time: vec![0;instance.m ],
            machine_state: vec![0;instance.m ],
            jobs: Rc::clone(&instance.jobs),
        }
    }

    fn map(&mut self, x: &Vec<usize>) -> CandidateSolution {
        self.machine_state.iter_mut().for_each(|x| *x = 0 );
        self.machine_time.iter_mut().for_each(|x| *x = 0 );
        self.job_time.iter_mut().for_each(|x| *x = 0);
        self.job_state.iter_mut().for_each(|x| *x = 0);
        let mut y = CandidateSolution::new(self.machine_time.len(), self.job_time.len());
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

struct SolutionSpace{
    instance: Rc<Instance>,
}

impl SolutionSpace{
    pub fn new(instance: Rc<Instance>) -> Self {
        Self{instance}
    }

    fn create(&self) -> CandidateSolution {
        CandidateSolution::new(self.instance.m, self.instance.n)
    }

    fn copy(from: CandidateSolution, mut to: CandidateSolution) where Self: Sized {
        to.schedule = from.schedule.clone();
    }
}

trait NullaryOperator{
    fn apply(&mut self, dest: &mut Vec<usize>, random: &mut ThreadRng);
}

struct SingleRandomSample {
    process: BlackBox,
    random: ThreadRng,

    best_makespan: usize,
    reset_counter: usize,
    reset_limit: usize,
    history: Vec<usize>,
}

impl TerminationCriterion for SingleRandomSample {
    fn should_terminate(&self) -> bool {
        if self.reset_counter % 1_000_0 == 0 {println!("{}", self.reset_counter)}
        return self.reset_counter >= self.reset_limit
    }
}

impl SingleRandomSample {
    pub fn new(black_box: BlackBox, random: ThreadRng) -> Self {
        Self {
            process: black_box,
            random,
            best_makespan: usize::MAX, reset_counter: 0, reset_limit: 1_000_00,
            history: vec![]
        }
    }

    fn solve(&mut self) -> Vec<usize> {
        let mut order = self.process.search_space.create();
        let mut schedule: CandidateSolution = self.process.mapping.map(&order);
        let mut random = thread_rng();

        while !self.should_terminate() {
            self.reset_counter += 1;
            self.apply(&mut order, &mut random);

            schedule = self.process.mapping.map(&order);
            let makespan = self.process.makespan.find_makespan(&schedule);
            if makespan < self.best_makespan {
                self.best_makespan = makespan;
                self.history.push(self.best_makespan);
                self.reset_counter = 0;
            }
        }
        self.process.save_history(&self.history, "random").expect("Failed to save.");
        self.process.save_schedule(&schedule, "random").expect("Failed to save Schedule.");
        order
    }
}

impl NullaryOperator for SingleRandomSample{
    fn apply(&mut self, dest: &mut Vec<usize>, random: &mut ThreadRng){
        dest.shuffle(random);
    }
}

fn main() {
    let instance: Instance = Instance::new("la01", "orlib");
    // let jssp: Rc<BlackBox> = Rc::new(BlackBox::new(instance));
    // let random: Rc<ThreadRng> = Rc::new(rand::thread_rng());

    let mut random_sample =
        SingleRandomSample::new(BlackBox::new(instance), thread_rng());

    random_sample.solve();
}