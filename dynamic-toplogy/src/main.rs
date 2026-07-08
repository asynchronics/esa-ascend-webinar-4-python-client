use nexosim::model::Model;
use nexosim::ports::{
    EventSource, Output, Requestor, SinkState, event_queue_endpoint, event_slot_endpoint,
};
use nexosim::server;
use nexosim::simulation::{Mailbox, SimInit};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Default)]
struct PowerSupply {
    pub power_out: Requestor<f64, f64>,
    pub power: Output<f64>,
}

#[Model]
impl PowerSupply {
    pub async fn voltage_setting(&mut self, voltage: f64) {
        if voltage < 0.0 {
            return;
        }

        let mut total_current = 0.0;
        for current in self.power_out.send(voltage).await {
            total_current += current;
        }

        self.power.send(voltage * total_current).await;
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Load {
    pub power: Output<f64>,
    resistance: f64,
}

#[Model]
impl Load {
    fn new(resistance: f64) -> Self {
        assert!(resistance > 0.0);

        Self {
            power: Output::default(),
            resistance,
        }
    }

    pub async fn power_in(&mut self, voltage: f64) -> f64 {
        let current = voltage / self.resistance;
        self.power.send(voltage * current).await;

        current
    }
}

#[derive(Deserialize)]
enum Topology {
    /// Topology with two different loads.
    AB(f64, f64),
    /// Topology with multiple identical loads
    Many(u8, f64),
}

fn simulation(topology: Topology) -> Result<SimInit, Box<dyn Error>> {
    let mut ps = PowerSupply::default();
    let ps_mbox = Mailbox::new();

    let mut bench = SimInit::new();

    EventSource::new()
        .connect(PowerSupply::voltage_setting, &ps_mbox)
        .bind_endpoint(&mut bench, "ps_voltage")?;

    let sink = event_slot_endpoint(&mut bench, SinkState::Enabled, "ps_power")?;
    ps.power.connect_sink(sink);

    match topology {
        Topology::AB(resistance_a, resistance_b) => {
            let mut load_a = Load::new(resistance_a);
            let load_a_mbox = Mailbox::new();

            let mut load_b = Load::new(resistance_b);
            let load_b_mbox = Mailbox::new();

            ps.power_out.connect(Load::power_in, &load_a_mbox);
            ps.power_out.connect(Load::power_in, &load_b_mbox);

            let sink = event_queue_endpoint(&mut bench, SinkState::Enabled, "power_A")?;
            load_a.power.connect_sink(sink);

            let sink = event_queue_endpoint(&mut bench, SinkState::Enabled, "power_B")?;
            load_b.power.connect_sink(sink);

            bench = bench.add_model(load_a, load_a_mbox, "Load_A").add_model(
                load_b,
                load_b_mbox,
                "Load_B",
            );
        }
        Topology::Many(n, resistance) => {
            let sink = event_queue_endpoint(&mut bench, SinkState::Enabled, "load_power")?;
            for i in (0..n).into_iter() {
                let mut load = Load::new(resistance);
                let load_mbox = Mailbox::new();

                ps.power_out.connect(Load::power_in, &load_mbox);
                load.power.connect_sink(sink.clone());

                bench = bench.add_model(load, load_mbox, &(format!("Load_{}", i + 1)));
            }
        }
    }

    Ok(bench.add_model(ps, ps_mbox, "POWER_SUPPLY"))
}

fn main() {
    server::run(simulation, "0.0.0.0:41633".parse().unwrap()).unwrap()
}
