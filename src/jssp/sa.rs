use crate::jssp::*;

pub struct SimulatedAnnealing {
    process: BlackBox,
    temperature_start: f64,
    annealing_speed: f64,
}

impl SimulatedAnnealing {
    pub fn new(instance: Instance, annealing_speed: f64, start_temperature: f64) -> Self {
        Self {
            process: BlackBox::new(instance, String::from("Simulated Annealing")),
            temperature_start: start_temperature,
            annealing_speed,
        }
    }

    pub fn solve(&mut self, temperature_operator: &str) -> BlackBox {
        let mut best_solution: Candidate = <BlackBox as NullaryOperator>::apply(&mut self.process);
        let mut curr: Candidate = best_solution.clone();
        let mut next: Candidate;

        let temperature_op: fn(&Self) -> f64 = match temperature_operator {
            "exponential" => <Self as TemperatureSchedule<Exponential>>::temperature,
            "logarithmic" => <Self as TemperatureSchedule<Logarithmic>>::temperature,
            _ => panic!("Unsupported temperature operator"),
        };

        while !(self.process.should_terminate)(&mut self.process) {
            next = <BlackBox as UnaryOperatorNSwap>::apply(&mut self.process, &curr);
            if next.makespan <= curr.makespan
                || self.process.random.gen_bool(((curr.makespan as i32 - next.makespan as i32) as f64
                / temperature_op(&self)).exp()) { curr = next; }

            if curr > best_solution {
                best_solution = curr.clone();
                self.process.update_history(&best_solution);
            }
        }

        self.process.update(&best_solution);
        let name = format!("annealing_{}", temperature_operator.to_lowercase());
        self.process.clone()
    }
}


struct Exponential;

struct Logarithmic;

trait TemperatureSchedule<T> { fn temperature(&self) -> f64; }

impl TemperatureSchedule<Exponential> for SimulatedAnnealing {
    fn temperature(&self) -> f64 {
        self.temperature_start * (1f64 - self.annealing_speed).powi(self.process.termination_counter as i32 - 1)
    }
}

impl TemperatureSchedule<Logarithmic> for SimulatedAnnealing {
    fn temperature(&self) -> f64 {
        self.temperature_start / ((self.process.termination_counter - 1) as f64 * self.annealing_speed + 1f64.exp()).ln()
    }
}