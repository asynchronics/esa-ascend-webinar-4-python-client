use std::error::Error;

use nexosim::{
    model::Model,
    server,
    simulation::{Mailbox, SimInit},
};
use schematic::{Config, ConfigLoader, Format};
use serde::{Deserialize, Serialize};

#[derive(Config, Debug, Serialize, Deserialize, Clone)]
struct GyroConfig {
    address: u8,
}

#[derive(Serialize, Deserialize)]
struct Gyro {
    address: u8,
}

#[Model]
impl Gyro {
    fn new(cfg: GyroConfig) -> Self {
        dbg!(&cfg);
        Self {
            address: cfg.address,
        }
    }
}

#[derive(Config, Debug, Serialize, Deserialize, Clone)]
struct ReactionWheelConfig {
    address: u8,
}

#[derive(Serialize, Deserialize)]
struct ReactionWheel {
    address: u8,
}

#[Model]
impl ReactionWheel {
    fn new(cfg: ReactionWheelConfig) -> Self {
        dbg!(&cfg);
        Self {
            address: cfg.address,
        }
    }
}

type BenchConfig = (String, String);

fn bench(cfg: BenchConfig) -> Result<SimInit, Box<dyn Error>> {
    let (gyro_cfg, rw_cfg) = cfg;

    let mut gyro_cfg_loader = ConfigLoader::<GyroConfig>::new();
    gyro_cfg_loader.code(gyro_cfg, Format::Toml)?;
    let gyro = Gyro::new(gyro_cfg_loader.load()?.config);

    let mut rw_cfg_loader = ConfigLoader::<ReactionWheelConfig>::new();
    rw_cfg_loader.code(rw_cfg, Format::Toml)?;
    let rw = ReactionWheel::new(rw_cfg_loader.load()?.config);

    Ok(SimInit::new()
        .add_model(gyro, Mailbox::new(), "GYRO")
        .add_model(rw, Mailbox::new(), "RW"))
}
fn main() {
    server::run(bench, "0.0.0.0:41633".parse().unwrap()).unwrap();
}
