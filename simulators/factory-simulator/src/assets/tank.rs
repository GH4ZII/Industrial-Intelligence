use super::MachineState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Tank {
    pub id: String,
    pub state: MachineState,
    pub level: f64,
    pub temperature: f64,
    pub pressure: f64,
}

impl Tank {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            level: 62.0,
            temperature: 35.0,
            pressure: 1.8,
            state: MachineState::Normal,
        }
    }

    pub fn update(&mut self) {
        match self.state {
            MachineState::Normal => {
                self.level = rand::random_range(55.0..75.0);
                self.temperature = rand::random_range(30.0..40.0);
                self.pressure = rand::random_range(1.5..2.2);
            }
            MachineState::Fault => {
                self.level = rand::random_range(5.0..20.0);
                self.temperature = rand::random_range(55.0..70.0);
                self.pressure = rand::random_range(3.5..5.0);
            }
        }
    }

    pub fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}
