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


fn main() {
    let termination_limit = 15;
    let is_timed = true;
    let instance_name = "ta02";
    let instance_type = InstanceType::TAILLARD;

    let instance = Instance::new(instance_name, instance_type, termination_limit, is_timed);
    let mut rs = RandomSample::new(instance.clone());
    let mut hc = HillClimber::new(&instance, 1_676, "nswap");
    // hc.solve_threaded().save_to_file();
    block_on(rs.solve_async()).save_to_file();
    // rs.solve().save_to_file();
    // rs.solve_threaded().save_to_file();
}
