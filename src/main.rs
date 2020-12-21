pub mod jssp;

use jssp::rs::RandomSample;
use jssp::hc::HillClimber;
use jssp::Instance;
use crate::jssp::UnaryOperator1Swap;
use crate::jssp::ga::Genetic;
use rand::seq::SliceRandom;
use rand::thread_rng;


fn main() {
    let instance = Instance::new("ta01", "taillard");
    let limit = 1_000_000;
    let mut random_sample =
        RandomSample::new(&instance, limit);
    let mut hill_climb_1swap =
        HillClimber::new(&instance, limit, 16_384);
    let mut hill_climb_nswap =
        HillClimber::new(&instance, limit, 16_384);
    let mut genetic =
        Genetic::new(&instance, limit);

    // random_sample.solve();
    // hill_climb_1swap.solve("1swap");
    // hill_climb_nswap.solve("nswap");
    genetic.solve(100,100);
    // let mut a = vec![1, 2, 3, 4, 5, 6, 7];
    // let mut r = thread_rng();
    // let (b, c) = a.split_at_mut(4);
    // b.shuffle(&mut r);
    // a = [b,c].concat();
    // println!("{:?}", a);
}
