use crate::jssp::*;
use std::thread::Thread;
use futures::task::{Context, Poll};
use std::pin::Pin;
use futures::{Stream, StreamExt};
use futures::executor::block_on;
use itertools::join;
use std::borrow::BorrowMut;


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

    pub fn solve(&mut self) -> BlackBox {
        let mut process = self.process.clone();
        let mut solution: Candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
        let mut best_solution = solution.clone();


        while !(self.process.should_terminate)(&mut process) {
            solution = <BlackBox as NullaryOperator>::apply(&mut process);
            process.update_history(&best_solution);

            if solution > best_solution { best_solution = solution }
        }
        process.update(&best_solution);

        process.finalize()
    }

    pub async fn solve_async(&mut self, should_save: bool) -> BlackBox {
        let mut gen = Generator::new(self.process.clone());
        let mut candidate: Candidate = gen.next().await.unwrap();

        // let timer = async { while !(self.process.should_terminate)(&mut self.process) {} };
        async_std::task::yield_now();

        let mut timer = Timer::new(self.process.clone());

        loop {
            gen.next().await.unwrap();
            if !timer.next().await.unwrap() { break; }
        };
        println!("{}", self.process.termination_counter);
        println!("{}", gen.process.termination_counter);
        self.process.clone()
    }
}

struct Generator { process: BlackBox }

impl Generator { fn new(process: BlackBox) -> Self { Self { process } } }

impl async_std::stream::Stream for Generator {
    type Item = Candidate;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.process.termination_counter += 1;
        Poll::Ready(Some(<BlackBox as NullaryOperator>::apply(&mut self.process)))
    }
}

struct Timer { process: BlackBox }

impl Timer { fn new(process: BlackBox) -> Self { Self { process } } }

impl async_std::stream::Stream for Timer {
    type Item = bool;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some((self.process.should_terminate)(self.process.borrow_mut())))
    }
}
