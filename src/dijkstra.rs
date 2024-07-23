use crate::Data;
use crate::Process;
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

#[derive(Clone, Eq, PartialEq, Debug)]
struct StockState(HashMap<String, u64>);

impl Hash for StockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in &self.0 {
            key.hash(state);
            value.hash(state);
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct State {
    time: u64,
    stocks: StockState,
    objective_count: u64,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.objective_count.cmp(&self.objective_count)
            .then_with(|| self.time.cmp(&other.time)) // Prioritize more objectives, then less time
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl State {
    fn new(time: u64, stocks: HashMap<String, u64>, objectives: &[String]) -> Self {
        let objective_count = objectives.iter()
            .map(|obj| *stocks.get(obj).unwrap_or(&0))
            .sum();
        State { time, stocks: StockState(stocks), objective_count }
    }
    
    fn apply_process(&self, process: &Process, objectives: &[String]) -> Option<Self> {
        let mut new_stocks = self.stocks.0.clone();

        for (input_item, input_amount) in &process.input {
            let entry = new_stocks.get_mut(input_item)?;
            if *entry < *input_amount {
                return None;
            }
            *entry -= *input_amount;
        }

        for (output_item, output_amount) in &process.output {
            *new_stocks.entry(output_item.clone()).or_insert(0) += *output_amount;
        }

        let objective_count = objectives.iter()
            .map(|obj| *new_stocks.get(obj).unwrap_or(&0))
            .sum();

        Some(State {
            time: self.time + process.time,
            stocks: StockState(new_stocks),
            objective_count,
        })
    }
}

pub fn optimize(data: Data) -> Option<(u64, HashMap<String, u64>)> {
    let mut heap = BinaryHeap::new();
    let mut visited = HashSet::new();
    let mut best_time = u64::MAX;
    let mut best_stocks = None;

    let optimize_for_time = data.objectives.contains(&"optimize".to_string());

    heap.push(State::new(0, data.stocks.clone(), &data.objectives));

    while let Some(state) = heap.pop() {
        if visited.contains(&state.stocks) {
            continue;
        }

        visited.insert(state.stocks.clone());

        if state.objective_count > 0 {
            if !optimize_for_time || state.time < best_time || best_stocks.is_none() {
                best_time = state.time;
                best_stocks = Some(state.stocks.0.clone());
            }
        }

        for process in &data.processes {
            if let Some(new_state) = state.apply_process(process, &data.objectives) {
                heap.push(new_state);
            }
        }
    }

    best_stocks.map(|stocks| (best_time, stocks))
}