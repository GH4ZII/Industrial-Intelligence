mod assets;
mod factory;

use factory::Factory;
use std::thread;
use std::time::Duration;

fn main() {
    let mut factory = Factory::new();

    loop {
        factory.tick();
        println!("{}", factory.to_json());
        thread::sleep(Duration::from_secs(1));
    }
}
