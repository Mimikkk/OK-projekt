#![feature(available_concurrency)]
#![feature(thread_id_value)]

pub mod jssp;

use jssp::rs::RandomSample;
use jssp::Instance;
use crate::jssp::rs::RandomSampleThreaded;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::prelude::*;
use serde::Serialize;
use crate::jssp::InstanceType;
use futures::executor::block_on;
use rand::{random, Rng};
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt, Future};
use std::thread;
use futures::channel::mpsc::{TryRecvError, SendError, Receiver};
use futures::task::{Poll, Context};
use crate::jssp::hc::HillClimber;
use itertools::Itertools;
use std::path::Path;
use std::fs::File;
use std::io::Write;


fn main() {
    let termination_limit = 10;
    let is_timed = true;
    let instance_name = "ta02";
    let instance_type = InstanceType::TAILLARD;

    let instance = Instance::new(instance_name, instance_type, termination_limit, is_timed);
    let mut rs = RandomSample::new(instance.clone());
    let mut hc = HillClimber::new(&instance, 1_676, "nswap");
    // hc.solve_threaded().save_to_file();
    let a = (1..=20)
        .map(|i| block_on(RandomSample::new(instance.clone()).solve_async(i)).termination_counter)
        .collect_vec();

    let data = serde_json::to_string_pretty(&a).expect("Failed to stringify");
    let mut fp = File::create(Path::new("experimental/async_performance.json")).expect("Failed to create file");
    write!(fp, "{}", data).expect("Failed to save data");
    // rs.solve().save_to_file();
    // rs.solve_threaded().save_to_file();
}
