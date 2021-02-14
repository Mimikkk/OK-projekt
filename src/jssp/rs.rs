use crate::jssp::*;
use std::thread::Thread;
use futures::task::{Context, Poll};
use std::pin::Pin;
use futures::{Stream, StreamExt};
use futures::executor::block_on;
use itertools::join;
use std::borrow::BorrowMut;
use std::thread;
use futures::channel::mpsc;


pub struct RandomSampleThreaded { instance: Instance }

impl RandomSampleThreaded {
    pub fn new(instance: &Instance) -> Self {
        Self { instance: instance.clone() }
    }
    pub fn solve(&self) -> BlackBox {
        let available_threads_count =
            std::thread::available_concurrency().map(|x| x.get()).unwrap_or(1);

        println!("Thread Count: {}", available_threads_count);
        let handles = (0..available_threads_count).map(|_| {
            let inst = self.instance.clone();
            std::thread::spawn(|| RandomSample::new(inst).solve())
        }).collect_vec();

        let bbs = handles.into_iter().map(|x| (x.thread().id().as_u64().get(), x.join().unwrap())).collect_vec();

        for (id, bb) in bbs.iter() {
            println!("Thread ID: {}", id);
            println!("Iter Count: {}", bb.termination_counter);
            println!("Best Candidate MS: {}", bb.best_candidate.makespan);
            println!("Time Elapsed: {}", bb.end_time);
            println!("Problem Lowerbound: {}", bb.lower_bound);
        }


        let best_solution =
            bbs.into_iter().max_by_key(|(_, bb)| bb.best_candidate.makespan).unwrap();
        println!("Best Thread: ID {} : Makespan {}", best_solution.0, best_solution.1.best_candidate.makespan);
        best_solution.1
    }
}

pub struct RandomSample { process: BlackBox }

impl RandomSample {
    pub fn new(instance: Instance) -> Self {
        Self { process: BlackBox::new(instance, String::from("random sample")) }
    }

    pub fn solve(&self) -> BlackBox {
        let mut process = self.process.clone();
        let mut solution: Candidate = <BlackBox as NullaryOperator>::apply(&mut process);
        let mut best_solution = solution.clone();


        while !(process.should_terminate)(&mut process) {
            solution = <BlackBox as NullaryOperator>::apply(&mut process);
            process.update_history(&best_solution);

            if solution > best_solution { best_solution = solution }
        }

        process.update(&best_solution);
        process.finalize()
    }

    pub fn solve_threaded(&self) -> BlackBox {
        let handles = (0..thread::available_concurrency().expect("Failed to get thread count").get())
            .map(|_| {
                let rs = RandomSample::new(self.process.instance.clone());
                thread::spawn(move || rs.solve())
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

    pub async fn solve_async(&mut self) -> BlackBox {
        let mut process = self.process.clone();

        // MPSC
        let (mut tx, mut rx) = mpsc::channel(1000);
        (0..100).for_each(|_| {
            let mut tx = tx.clone();
            let mut gen_process = self.process.clone();
            thread::spawn(move || {
                while !tx.is_closed() {
                    tx.start_send(<BlackBox as NullaryOperator>::apply(&mut gen_process));
                }
            });
        });

        let mut solution: Candidate = rx.next().await.expect("Failed to get next Candidate");
        let mut best_solution = solution.clone();

        while !(self.process.should_terminate)(&mut process) {
            solution = rx.next().await.expect("Failed to get next Candidate");

            process.update_history(&best_solution);
            if solution > best_solution { best_solution = solution }
        }

        process.update(&best_solution);
        process.finalize()
    }
}
