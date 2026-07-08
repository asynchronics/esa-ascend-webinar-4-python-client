use std::{error::Error, time::Duration};

use nexosim::{
    model::{Context, Model, schedulable},
    ports::{Output, SinkState, event_queue_endpoint},
    server,
    simulation::{Mailbox, SimInit},
};
use serde::{Deserialize, Serialize};

/// Dynamics update period.
const UPDATE_PERIOD: Duration = Duration::from_millis(100);

#[derive(Serialize, Deserialize, Default)]
struct Gyroscope {
    yaw: Output<f32>,
    pitch: Output<f32>,
    roll: Output<f32>,
}

#[Model]
impl Gyroscope {
    #[nexosim(schedulable)]
    async fn dynamics_update(&mut self, _: (), ctx: &Context<Self>) {
        let t = ctx.time().as_secs() as f32 + ctx.time().subsec_nanos() as f32 * 1e-9;

        self.yaw.send(t.cos()).await;
        self.pitch.send(t.sin()).await;
        self.roll.send(t.cos()).await;
    }

    #[nexosim(init)]
    async fn init(&mut self, ctx: &Context<Self>) {
        // Schedule periodic dynamics update
        ctx.schedule_periodic_event(
            UPDATE_PERIOD,
            UPDATE_PERIOD,
            schedulable!(Gyroscope::dynamics_update),
            (),
        )
        .unwrap();
    }
}

fn simulation(_: ()) -> Result<SimInit, Box<dyn Error>> {
    let mut gyro = Gyroscope::default();
    let mbox = Mailbox::new();

    let mut bench = SimInit::new();

    let yaw = event_queue_endpoint(&mut bench, SinkState::Enabled, "yaw")?;

    // Retrieve a clock reader and move it to the mapping closure.
    let clock = bench.clock_reader();
    gyro.yaw.map_connect_sink(move |e| (clock.time(), *e), yaw);

    let pitch = event_queue_endpoint(&mut bench, SinkState::Enabled, "pitch")?;
    let clock = bench.clock_reader();
    gyro.pitch
        .map_connect_sink(move |e| (clock.time(), *e), pitch);

    let roll = event_queue_endpoint(&mut bench, SinkState::Enabled, "roll")?;
    let clock = bench.clock_reader();
    gyro.roll
        .map_connect_sink(move |e| (clock.time(), *e), roll);

    Ok(bench.add_model(gyro, mbox, "GYRO"))
}

fn main() {
    server::run(simulation, "0.0.0.0:41633".parse().unwrap()).unwrap()
}
