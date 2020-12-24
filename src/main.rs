pub mod jssp;

use jssp::rs::RandomSample;
use jssp::hc::HillClimber;
use jssp::Instance;
use crate::jssp::ga::Genetic;

fn main() {
    let instance = Instance::new("ta01", "taillard");
    let limit = 100;
    let mut random_sample =
        RandomSample::new(&instance, limit);

    let mut hill_climb =
        HillClimber::new(&instance, limit, 16_384);

    let mut genetic =
        Genetic::new(&instance, limit);

    random_sample.solve();
    hill_climb.solve("1swap");
    hill_climb.solve("nswap");
    genetic.solve(0.7, 8192, 8192);
}
