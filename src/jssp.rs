use itertools::{Itertools, zip};
use std::cmp::{max, min};
use std::io::{BufReader, BufRead, Write};
use std::path::Path;
use std::fs::File;
use std::rc::Rc;
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

pub mod rs;
pub mod hc;
pub mod ga;

trait TerminationCriterion {
    fn should_terminate(&self) -> bool;
}

trait NullaryOperator {
    fn apply(&self, random: &mut ThreadRng) -> Vec<usize>;
}

pub trait UnaryOperator1Swap {
    fn apply(&self, based: &Vec<usize>, random: &mut ThreadRng) -> Vec<usize>{
        let mut result = based.clone();
        let high = based.len();
        let i: usize = random.gen_range(0, high);
        let mut j: usize = random.gen_range(0, high);
        while result[i] == result[j] { j = random.gen_range(0, high) }
        result.swap(i,j);
        result

    }
}

pub trait UnaryOperatorNSwap {
    fn apply(&self, based: &Vec<usize>, random: &mut ThreadRng) -> Vec<usize> {
        let mut result = based.clone();
        let high = based.len();
        let mut should_flip = true;
        let mut i: usize;
        let mut j: usize;

        while should_flip {
            i = random.gen_range(0, high);
            j = random.gen_range(0, high);
            while result[i] == result[j] { j = random.gen_range(0, high) }
            result.swap(i,j);

            should_flip = random.gen();
        }
        result
    }
}

trait BinaryOperator {
    fn apply(&self, based_a: &Vec<usize>, based_b: &Vec<usize>,
                random: &mut ThreadRng) -> Vec<usize>;
}

#[derive(Clone)]
pub struct Instance {
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
        let mut _b: usize=0;
        let mut times: Vec<Vec<usize>> = Vec::new();
        let mut machines: Vec<Vec<usize>> = Vec::new();
        for (i, line) in contents.lines().skip(1)
            .map(|x| x.expect("line wasn't read properly")).enumerate() {
            if i == 0 {
                let vec: Vec<usize> = line.split_whitespace()
                    .map(|x| x.parse().expect("Not parse the number.")).collect_vec();
                a = vec[0];
                _b = vec[1];
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

#[derive(Clone)]
pub struct CandidateSolution {
    pub schedule: Vec<Vec<usize>>,
}

impl CandidateSolution {
    fn new(m: usize, n: usize) -> Self{
        Self{schedule: vec![vec![0 ; 3*n] ; m]}
    }
}

#[derive(Clone)]
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

    fn copy(from: Vec<usize>, mut _to: Vec<usize>) {
        _to = from.clone();
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

impl RepresentationMapping{
    fn new(instance: Rc<Instance>) -> Self {
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

        let mut machine: usize = 0;
        let mut job_step: usize = 0;

        let mut start: usize = 0;
        let mut end: usize = 0;

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
struct SolutionSpace{
    instance: Rc<Instance>,
}

impl SolutionSpace{
    fn new(instance: Rc<Instance>) -> Self {
        Self{instance}
    }

    fn create(&self) -> CandidateSolution {
        CandidateSolution::new(self.instance.m, self.instance.n)
    }

    fn copy(from: CandidateSolution, mut to: CandidateSolution) where Self: Sized {
        to.schedule = from.schedule.clone();
    }
}

#[derive(Clone)]
pub struct BlackBox {
    instance: Rc<Instance>,
    search_space: SearchSpace,
    mapping: RepresentationMapping,

    pub best_solution: CandidateSolution,
    pub best_makespan: usize,
    pub best_order: Vec<usize>,
    pub history: Vec<usize>,

    pub lower_bound: usize,
    pub upper_bound: usize,
}

impl BlackBox {
    fn new(instance: Instance) -> Self{
        let p_instance = Rc::new(instance);
        let search_space = SearchSpace::new(Rc::clone(&p_instance));
        let mut mapping = RepresentationMapping::new(Rc::clone(&p_instance));
        let best_order = search_space.create();
        let best_solution = mapping.map(&best_order);

        let mut bb = Self {
            instance: p_instance,
            search_space,
            mapping,

            best_solution,
            best_makespan: usize::MAX,
            best_order: Vec::new(),
            history: Vec::new(),

            lower_bound: 0,
            upper_bound: 0,
        };
        bb.lower_bound = bb.find_lower_bound();
        bb.upper_bound = bb.find_upper_bound();
        bb
    }
    fn update(&mut self, order: &Vec<usize>){
        self.best_order = order.clone();
        self.best_solution = self.mapping.map(&order);
        self.best_makespan = self.find_makespan(&self.best_solution);
        self.update_history(self.best_makespan);
    }

    fn update_history(&mut self, makespan: usize){
        self.history.push(makespan);
    }

    fn save(&self, name: &str) -> std::io::Result<()> {
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

    fn save_schedule(&self, name: &str) -> std::io::Result<()> {
        let s = format!("solutions\\{}_{}_solution.txt", self.instance.id, name);
        let path = Path::new(s.as_str());
        let mut file = File::create(path)?;

        writeln!(file, "{} {}", self.instance.n, self.instance.m)?;
        writeln!(file, "{}", self.best_makespan)?;
        for line in self.best_solution.schedule.iter() {
            for i in line{ write!(file, "{} ", i)? }
            write!(file, "\n")?;
        }

        Ok(())
    }

    fn find_makespan(&self, y: &CandidateSolution) -> usize {
        y.schedule.iter().map(|x| *x.last().unwrap()).max().expect("Failed to find makespan.")
    }

    fn find_lower_bound(&mut self) -> usize {
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

    fn find_upper_bound(&self) -> usize{
        //Sloppy.
        self.instance.jobs.iter().map(|j| (1..j.len()).step_by(2).rev().map(|x| j[x]).sum::<usize>()).sum()
    }

}
