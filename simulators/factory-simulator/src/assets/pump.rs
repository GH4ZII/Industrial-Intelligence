use super::MachineState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Pump {
    pub id: String,
    pub state: MachineState,
    pub rpm: f64,
    pub temperature: f64,
    pub vibration: f64,
    pub pressure: f64,
    pub power: f64,
}

impl Pump {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            rpm: 1450.0,
            temperature: 63.0,
            vibration: 2.1,
            pressure: 5.2,
            power: 12.4,
            state: MachineState::Normal,
        }
    }

    pub fn update(&mut self) {
        match self.state {
            MachineState::Normal => {
                self.rpm = rand::random_range(1430.0..1470.0);
                self.temperature = rand::random_range(60.0..68.0);
                self.vibration = rand::random_range(1.5..3.0);
                self.pressure = rand::random_range(4.8..5.5);
                self.power = rand::random_range(11.0..14.0);
            }
            MachineState::Fault => {
                self.rpm = rand::random_range(1150.0..1300.0);
                self.temperature = rand::random_range(95.0..115.0);
                self.vibration = rand::random_range(7.0..11.0);
                self.pressure = rand::random_range(2.5..4.0);
                self.power = rand::random_range(15.0..20.0);
            }
        }
    }

    pub fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}
