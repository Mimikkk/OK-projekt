pub mod jssp;

use jssp::srs::SingleRandomSample;
use jssp::shc::SingleHillClimber;
use jssp::Instance;

fn main() {
    let instance = Instance::new("ta01", "taillard");
    let limit = 1_000_0;
    let mut random_sample =
        SingleRandomSample::new(&instance, limit);
    let mut single_sample =
        SingleHillClimber::new(&instance, limit, 16_384);
    // random_sample.solve();
    single_sample.solve();
}
