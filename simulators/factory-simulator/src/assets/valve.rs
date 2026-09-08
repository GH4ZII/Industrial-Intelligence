use super::MachineState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Valve {
    pub id: String,
    pub state: MachineState,
    pub open: bool,
    pub flow: f64,
    pub position: f64,
}

impl Valve {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            open: true,
            flow: 12.0,
            position: 100.0,
            state: MachineState::Normal,
        }
    }

    pub fn update(&mut self) {
        match self.state {
            MachineState::Normal => {
                self.open = true;
                self.position = rand::random_range(95.0..100.0);
                self.flow = rand::random_range(10.0..14.0);
            }
            MachineState::Fault => {
                self.open = false;
                self.position = rand::random_range(0.0..15.0);
                self.flow = rand::random_range(0.0..2.0);
            }
        }
    }

    pub fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}
