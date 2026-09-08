mod motor;
mod pump;
mod tank;
mod valve;

pub use motor::Motor;
pub use pump::Pump;
pub use tank::Tank;
pub use valve::Valve;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum MachineState {
    Normal,
    Fault,
}
