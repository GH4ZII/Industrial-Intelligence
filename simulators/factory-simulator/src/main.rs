use serde::Serialize;
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize)]
enum MachineState {
    Normal,
    Fault,
}

#[derive(Debug, Serialize)]
struct Pump {
    id: String,
    state: MachineState,
    rpm: f64,
    temperature: f64,
    vibration: f64,
    pressure: f64,
    power: f64,
}

#[derive(Debug, Serialize)]
struct Motor {
    id: String,
    state: MachineState,
    temperature: f64,
    rpm: f64,
    current: f64,
    vibration: f64,
}

#[derive(Debug, Serialize)]
struct Tank {
    id: String,
    state: MachineState,
    level: f64,
    temperature: f64,
    pressure: f64,
}

#[derive(Debug, Serialize)]
struct Valve {
    id: String,
    state: MachineState,
    open: bool,
    flow: f64,
    position: f64,
}

impl Pump {
    fn new(id: &str) -> Self {
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

    fn update(&mut self) {
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

    fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}

impl Motor {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            temperature: 55.0,
            rpm: 1500.0,
            current: 18.0,
            vibration: 1.8,
            state: MachineState::Normal,
        }
    }

    fn update(&mut self) {
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

    fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}

impl Tank {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            level: 62.0,
            temperature: 35.0,
            pressure: 1.8,
            state: MachineState::Normal,
        }
    }

    fn update(&mut self) {
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

    fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}

impl Valve {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            open: true,
            flow: 12.0,
            position: 100.0,
            state: MachineState::Normal,
        }
    }

    fn update(&mut self) {
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

    fn maybe_trigger_fault(&mut self) {
        if rand::random_range(0.0..1.0) < 0.02 {
            self.state = MachineState::Fault;
        }
    }
}

#[derive(Serialize)]
struct FactorySnapshot<'a> {
    pump: &'a Pump,
    motor: &'a Motor,
    tank: &'a Tank,
    valve: &'a Valve,
}

fn main() {
    let mut pump = Pump::new("P-101");
    let mut motor = Motor::new("M-101");
    let mut tank = Tank::new("T-101");
    let mut valve = Valve::new("V-101");

    loop {
        pump.maybe_trigger_fault();
        motor.maybe_trigger_fault();
        tank.maybe_trigger_fault();
        valve.maybe_trigger_fault();

        pump.update();
        motor.update();
        tank.update();
        valve.update();

        let snapshot = FactorySnapshot {
            pump: &pump,
            motor: &motor,
            tank: &tank,
            valve: &valve,
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        println!("{json}");

        thread::sleep(Duration::from_secs(1));
    }
}
