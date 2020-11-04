pub mod jssp;

use jssp::rs::RandomSample;
use jssp::hc::HillClimber;
use jssp::Instance;
use crate::jssp::UnaryOperator1Swap;


fn main() {
    let instance = Instance::new("ta01", "taillard");
    let limit = 1_000_000_0;
    let mut random_sample =
        RandomSample::new(&instance, limit);
    let mut single_sample1 =
        HillClimber::new(&instance, limit, 16_384);
    let mut single_sample2 =
        HillClimber::new(&instance, limit, 16_384);
    random_sample.solve();
    // single_sample1.solve("1swap");
    // single_sample2.solve("nswap");
}
