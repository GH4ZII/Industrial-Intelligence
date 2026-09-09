use crate::assets::{Motor, Pump, Tank, Valve};
use serde::Serialize;

#[derive(Serialize)]
pub struct Factory {
    pump: Pump,
    motor: Motor,
    tank: Tank,
    valve: Valve,
}

impl Factory {
    pub fn new() -> Self {
        Self {
            pump: Pump::new("P101"),
            motor: Motor::new("M201"),
            tank: Tank::new("T301"),
            valve: Valve::new("V401"),
        }
    }

    pub fn pump(&self) -> &Pump {
        &self.pump
    }

    pub fn motor(&self) -> &Motor {
        &self.motor
    }

    pub fn tank(&self) -> &Tank {
        &self.tank
    }

    pub fn valve(&self) -> &Valve {
        &self.valve
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
        serde_json::to_string_pretty(self).expect("factory should always serialize")
    }
}
