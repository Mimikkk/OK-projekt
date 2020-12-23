pub mod jssp;
#[allow(dead_code)]
use jssp::rs::RandomSample;
use jssp::hc::HillClimber;
use jssp::Instance;
use crate::jssp::ga::Genetic;

fn main() {
    let instance = Instance::new("ta01", "taillard");
    let limit = 1_000_0;
    let mut random_sample =
        RandomSample::new(&instance, limit);

    let mut hill_climb_1swap =
        HillClimber::new(&instance, limit, 16_384);

    let mut hill_climb_nswap =
        HillClimber::new(&instance, limit, 16_384);

    let mut genetic =
        Genetic::new(&instance, limit);

    genetic.solve(0.7,8192,8192);
}
