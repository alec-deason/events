use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::fmt::Debug;

use rand::distributions::{Exp, Distribution};

use ordered_float::OrderedFloat;

pub type State = HashMap<Place, PlaceState>;
pub type Place = usize;
pub type Time = f64;

#[derive(Debug, Copy, Clone)]
pub struct PlaceState {
    pub tokens: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct StateChange {
    pub place: Place,
    pub value: i32,
}

pub trait Event: Debug {
    fn enablement_inputs(&self) -> Vec<Place>;
    fn rate_inputs(&self) -> Vec<Place>;
    fn outputs(&self) -> Vec<Place>;

    fn enabled(&self, inputs: &[PlaceState]) -> bool { true }
    fn hazard_rate(&self, inputs: &[PlaceState]) -> f64;
    fn fire(&self) -> Vec<StateChange>;
}

#[derive(Debug)]
pub struct Simulation {
    pub state: State,
    current_time: Time,
    events: Vec<Box<dyn Event>>,
    valid_firing: Vec<usize>,
    dependencies: HashMap<Place, HashSet<usize>>,
    upcoming_firings: BinaryHeap<ScheduledFiring>,
}

#[derive(Debug, Copy, Clone)]
struct ScheduledFiring(Time, usize, usize);

impl Ord for ScheduledFiring {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip order to make `upcoming_firings` behave as a min heap
        OrderedFloat(other.0).cmp(&OrderedFloat(self.0))
    }
}

impl PartialOrd for ScheduledFiring {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledFiring {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 &&
            //This is only valid when comparing objects from the same simulation but that's
            //probably fine...
            self.1 == other.1 &&
            self.2 == other.2
    }
}
impl Eq for ScheduledFiring {}


impl Simulation {
    //TODO: This is conspicuously missing a good way to set initial state

    pub fn from_events(events: Vec<Box<dyn Event>>) -> Self {
        //TODO: I know you can do this by chaining iterators but I can't make it work
        let mut places = Vec::new();
        let mut dependencies = HashMap::new();
        for (event_idx, event) in events.iter().enumerate() {
            let mut this_event_places = HashSet::new();
            this_event_places.extend(event.enablement_inputs());
            this_event_places.extend(event.rate_inputs());
            for place in &this_event_places {
                dependencies.entry(*place).or_insert(HashSet::new()).insert(event_idx);
            }
            this_event_places.extend(event.outputs());
            places.extend(this_event_places);
        }
        let state = places.iter().map(|p| (*p, PlaceState { tokens: 0 })).collect();
        Self {
            state,
            current_time: 0.0,
            valid_firing: events.iter().map(|_| 0).collect(),
            events: events,
            dependencies,
            upcoming_firings: BinaryHeap::new(),
        }
    }

    pub fn schedule_event(&mut self, event_idx: usize) {
        let event = &self.events[event_idx];
        let enablement_state: Vec<_> = event.enablement_inputs().iter().map(|p| self.state[p]).collect();
        self.valid_firing[event_idx] += 1;
        if event.enabled(&enablement_state) {
            let rate_state: Vec<_> = event.rate_inputs().iter().map(|p| self.state[p]).collect();
            let rate = event.hazard_rate(&rate_state);
            let firing_time =  Exp::new(rate).sample(&mut rand::thread_rng()) + self.current_time;
            self.upcoming_firings.push(ScheduledFiring(firing_time, event_idx, self.valid_firing[event_idx]));
        }
    }

    pub fn setup_initial_firings(&mut self) {
        for event_idx in 0..self.events.len() {
            self.schedule_event(event_idx);
        }
    }

    fn pop_event(&mut self) -> Option<ScheduledFiring> {
        let mut potential_next = self.upcoming_firings.pop();
        while let Some(next) = &potential_next {
            if self.valid_firing[next.1] == next.2 {
                break
            }
            potential_next = self.upcoming_firings.pop();
        }
        potential_next
    }

    pub fn run_until(&mut self, time: Time) {
        let mut potential_next = self.pop_event();
        while let Some(next) = potential_next {
            if next.0 > time {
                self.upcoming_firings.push(next);
                break
            }

            self.current_time = next.0;

            let event = &self.events[next.1];
            let mut events_to_reschedule = HashSet::new();
            events_to_reschedule.insert(next.1);
            for state_change in event.fire() {
                if let Some(events) = self.dependencies.get(&state_change.place) {
                    events_to_reschedule.extend(events);
                }
                self.state.get_mut(&state_change.place).unwrap().tokens += state_change.value;
            }
            for event_idx in events_to_reschedule {
                self.schedule_event(event_idx);
            }

            potential_next = self.pop_event();
        }
    }
}
