use crate::assets::{Motor, Pump, Tank, Valve};
use serde::Serialize;

#[derive(Serialize)]
pub struct Factory {
    pub pump: Pump,
    pub motor: Motor,
    pub tank: Tank,
    pub valve: Valve,
}

impl Factory {
    pub fn new() -> Self {
        Self {
            pump: Pump::new("P-101"),
            motor: Motor::new("M-101"),
            tank: Tank::new("T-101"),
            valve: Valve::new("V-101"),
        }
    }

    pub fn tick(&mut self) {
        self.pump.maybe_trigger_fault();
        self.motor.maybe_trigger_fault();
        self.tank.maybe_trigger_fault();
        self.valve.maybe_trigger_fault();

        self.pump.update();
        self.motor.update();
        self.tank.update();
        self.valve.update();
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("factory should always serialize")
    }
}
