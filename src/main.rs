pub mod jssp;

use jssp::rs::RandomSample;
use jssp::hc::HillClimber;
use jssp::Instance;
use jssp::ga::Genetic;
use crate::jssp::sa::SimulatedAnnealing;

fn main() {
    let termination_limit = 60;
    let is_timed = true;
    let instance_name = "ta01";
    let instance_type = "taillard";

    let instance = Instance::new(instance_name, instance_type, termination_limit, is_timed);

    let mut random_sample = RandomSample::new(&instance);
    let mut hill_climb_1swap = HillClimber::new(&instance, 16_384);
    let mut hill_climb_nswap = HillClimber::new(&instance, 16_384);
    let mut genetic = Genetic::new(&instance);

    let mut annealing_logarithmic =
        SimulatedAnnealing::new(&instance, 1.1817e-7, 21.7);
    let mut annealing_exponential =
        SimulatedAnnealing::new(&instance, 1.1817e-7, 21.7);

    random_sample.solve();
    // hill_climb_1swap.solve("1swap");
    // hill_climb_nswap.solve("nswap");
    // genetic.solve(0.7, 8192, 8192);

    // annealing_logarithmic.solve("logarithmic");
    // annealing_exponential.solve("exponential");
}
