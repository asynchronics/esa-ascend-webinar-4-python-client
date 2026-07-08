use std::time::Duration;

use db_adapter::{DbAdapter, ProtoDb};
use nexosim::{
    model::{Context, Model, schedulable},
    ports::Output,
    simulation::{Mailbox, SimInit},
    time::{AutoSystemClock, MonotonicTime, PeriodicTicker},
};
use serde::{Deserialize, Serialize};

const CONNECTION_STRING: &str = "postgresql://postgres:123@localhost:5432/nexosim";

#[derive(Serialize, Deserialize, Default)]
struct MyModel {
    out: Output<f32>,
}

#[Model]
impl MyModel {
    #[nexosim(schedulable)]
    async fn send_sine(&mut self, _: (), ctx: &Context<Self>) {
        let t = ctx
            .time()
            .duration_since(MonotonicTime::EPOCH)
            .as_secs_f32();

        self.out.send(t.sin()).await;
    }

    #[nexosim(init)]
    async fn init(&mut self, ctx: &Context<Self>) {
        ctx.schedule_periodic_event(
            Duration::from_millis(100),
            Duration::from_millis(100),
            schedulable!(MyModel::send_sine),
            (),
        )
        .unwrap();
    }
}

pub fn bench() -> Result<SimInit, Box<dyn std::error::Error>> {
    let mut model = MyModel::default();
    let mbox = Mailbox::new();

    let db = ProtoDb::new(CONNECTION_STRING.to_string());
    let db_mbox = Mailbox::new();

    model.out.map_connect(
        |e| format!("INSERT INTO sine (val) VALUES ({});", e),
        DbAdapter::execute_query,
        &db_mbox,
    );

    let bench = SimInit::new()
        .add_model(model, mbox, "MODEL")
        .add_model(db, db_mbox, "Db Adapter")
        .with_clock(
            AutoSystemClock::new(),
            PeriodicTicker::new(Duration::from_millis(50)),
        );

    Ok(bench)
}

fn main() {
    let mut sim = bench().unwrap().init(MonotonicTime::EPOCH).unwrap();

    sim.step_until(Duration::from_millis(500)).unwrap();

    if let Some(mut client) = postgres::Client::connect(&CONNECTION_STRING, postgres::NoTls).ok() {
        let reply = client.query("SELECT * FROM sine;", &[]).unwrap();

        for item in reply.iter() {
            println!("{:?}", item.get::<&str, f32>("val"));
        }
    }
}
