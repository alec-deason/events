use std::f64::INFINITY;

use events::*;

#[derive(Debug, Copy, Clone)]
struct SimpleTransition {
    input_place: Place,
    output_place: Place,
    rate: f64,
}

impl Event for SimpleTransition {
    fn enablement_inputs(&self) -> Vec<Place> {
        vec![self.input_place]
    }

    fn rate_inputs(&self) -> Vec<Place> {
        vec![]
    }

    fn outputs(&self) -> Vec<Place> {
        vec![self.output_place, self.input_place]
    }
            
    fn enabled(&self, inputs: &[PlaceState]) -> bool {
        inputs[0].tokens > 0
    }

    fn hazard_rate(&self, _inputs: &[PlaceState]) -> f64 {
        self.rate
    }

    fn fire(&self) -> Vec<StateChange> {
        vec![
            StateChange {
                place: self.input_place,
                value: -1
            },
            StateChange {
                place: self.output_place,
                value: 1
            }
        ]
    }

}

fn main() {
    let place_a = 0;
    let place_b = 1;
    let place_c = 2;
    let event_one = Box::new(SimpleTransition {
        input_place: place_a,
        output_place: place_b,
        rate: 0.01,
    });
    let event_two = Box::new(SimpleTransition {
        input_place: place_a,
        output_place: place_c,
        rate: 0.02,
    });

    let mut sim = Simulation::from_events(vec![event_one, event_two]);

    // Setup initial marking
    sim.state.get_mut(&place_a).unwrap().tokens += 10;
    sim.setup_initial_firings();

    println!("{:#?}", sim);

    sim.run_until(INFINITY.into());

    println!("{:#?}", sim);
}
