use super::MachineState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Motor {
    pub id: String,
    pub state: MachineState,
    pub temperature: f64,
    pub rpm: f64,
    pub current: f64,
    pub vibration: f64,
}

impl Motor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            temperature: 55.0,
            rpm: 1500.0,
            current: 18.0,
            vibration: 1.8,
            state: MachineState::Normal,
        }
    }

    pub fn update(&mut self) {
        match self.state {
            MachineState::Normal => {
                self.temperature = rand::random_range(50.0..62.0);
                self.rpm = rand::random_range(1480.0..1520.0);
                self.current = rand::random_range(16.0..20.0);
                self.vibration = rand::random_range(1.2..2.5);
            }
            MachineState::Fault => {
                self.temperature = rand::random_range(90.0..110.0);
                self.rpm = rand::random_range(1100.0..1300.0);
                self.current = rand::random_range(28.0..40.0);
                self.vibration = rand::random_range(6.0..12.0);
            }
        }
    }

    pub fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}
