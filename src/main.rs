pub mod jssp;

use jssp::rs::RandomSample;
use jssp::hc::HillClimber;
use jssp::Instance;
use jssp::ga::Genetic;
use crate::jssp::tabu::Tabu;

fn main() {
    let instance = Instance::new("ta01", "taillard");
    let limit = 1000;
    let mut random_sample =
        RandomSample::new(&instance, limit);

    let mut hill_climb_1swap =
        HillClimber::new(&instance, limit, 16_384);

    let mut hill_climb_nswap =
        HillClimber::new(&instance, limit, 16_384);

    let mut genetic =
        Genetic::new(&instance, limit);
    let mut tabu = Tabu::new(&instance, 50, limit);

    // tabu.solve();
    // random_sample.solve();
    // hill_climb_1swap.solve("1swap");
    // hill_climb_nswap.solve("nswap");
    genetic.solve(0.7, 8192, 8192);
}
