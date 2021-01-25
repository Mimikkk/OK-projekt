#![feature(available_concurrency)]
#![feature(thread_id_value)]

pub mod jssp;

use jssp::rs::RandomSample;
use jssp::Instance;
use crate::jssp::rs::RandomSampleThreaded;

async fn hewwo() { eprintln!("Hewwo") }

fn main() {
    // Config
    let termination_limit = 10;
    let is_timed = true;
    let instance_name = "ta02";
    let instance_type = "taillard";

    let instance = Instance::new(instance_name, instance_type, termination_limit, is_timed);
    let mut rs = RandomSample::new(instance.clone());

    let bb = async_std::task::block_on(rs.solve_async(false));
    println!("Iter Count: {}", bb.termination_counter);
    println!("Best Candidate MS: {}", bb.best_candidate.makespan);
    println!("Time Elapsed: {}", bb.prev_save);
    println!("Problem Lowerbound: {}", bb.lower_bound);
}
