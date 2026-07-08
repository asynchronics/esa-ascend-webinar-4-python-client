use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use nexosim::model::{Context, ProtoModel, schedulable};
use nexosim::ports::{EventSource, Output};
use nexosim::server;
use nexosim::simulation::{AutoEventKey, Mailbox, SimInit};
use nexosim::{Message, model::Model};
use serde::{Deserialize, Serialize};

const OBC_ADDR: u8 = 0x11;
const DEVICE_ADDR: u8 = 0x02;

#[derive(Clone, Message, Serialize, Deserialize, Debug)]
struct Packet {
    src_address: u8,
    dest_address: u8,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Default)]
struct Obc {
    tc: Output<Packet>,
    device_timeout_key: Option<AutoEventKey>,
}

#[Model]
impl Obc {
    async fn send_tc(&mut self, data: Vec<u8>, ctx: &Context<Self>) {
        self.tc
            .send(Packet {
                src_address: OBC_ADDR,
                dest_address: DEVICE_ADDR,
                data,
            })
            .await;

        // Schedule a timeout event to happen in 10 milliseconds.
        self.device_timeout_key = Some(
            ctx.schedule_keyed_event(
                Duration::from_millis(10),
                schedulable!(Obc::device_timeout),
                (),
            )
            .unwrap()
            .into(),
        );
    }

    fn tm(&mut self, packet: Packet) {
        // Cancel timeout event if a reply was received.
        if packet.src_address == DEVICE_ADDR {
            self.device_timeout_key.take();
        }
        dbg!(packet);
    }

    #[nexosim(schedulable)]
    fn device_timeout(&mut self) {
        println!("Device TC timed out.")
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Device {
    tm: Output<Packet>,
}

#[Model]
impl Device {
    async fn tc(&mut self, packet: Packet) {
        self.tm
            .send(Packet {
                src_address: DEVICE_ADDR,
                dest_address: packet.src_address,
                data: packet.data,
            })
            .await;
    }
}

struct FaultManagerEnv {
    tmtc_switch: Arc<AtomicBool>,
}

#[derive(Serialize, Deserialize)]
struct FaultManager;

#[Model(type Env=FaultManagerEnv)]
impl FaultManager {
    fn toggle_tmtc_fault(
        &mut self,
        state: bool,
        _: &Context<Self>,
        env: &mut <Self as Model>::Env,
    ) {
        env.tmtc_switch.store(state, Ordering::Relaxed);
    }
}

struct ProtoFaultManager {
    tmtc_switch: Arc<AtomicBool>,
}

impl ProtoModel for ProtoFaultManager {
    type Model = FaultManager;

    fn build(
        self,
        _: &mut nexosim::model::BuildContext<Self>,
    ) -> (Self::Model, <Self::Model as Model>::Env) {
        (
            FaultManager,
            FaultManagerEnv {
                tmtc_switch: self.tmtc_switch,
            },
        )
    }
}

fn simulation(_: ()) -> Result<SimInit, Box<dyn Error>> {
    let mut obc = Obc::default();
    let obc_mbox = Mailbox::new();

    let mut device = Device::default();
    let device_mbox = Mailbox::new();

    let switch = Arc::new(AtomicBool::new(false));

    let switch_clone = switch.clone();
    obc.tc.filter_map_connect(
        move |e| {
            if switch_clone.load(Ordering::Relaxed) {
                None
            } else {
                Some(e.clone())
            }
        },
        Device::tc,
        &device_mbox,
    );

    device.tm.connect(Obc::tm, &obc_mbox);

    let fault_manager = ProtoFaultManager {
        tmtc_switch: switch,
    };
    let fm_mbox = Mailbox::new();

    let mut bench = SimInit::new();

    EventSource::new()
        .connect(FaultManager::toggle_tmtc_fault, &fm_mbox)
        .bind_endpoint(&mut bench, "toggle_tmtc_fault")?;

    EventSource::new()
        .connect(Obc::send_tc, &obc_mbox)
        .bind_endpoint(&mut bench, "obc_send_tc")?;

    Ok(bench
        .add_model(obc, obc_mbox, "OBC")
        .add_model(device, device_mbox, "DEVICE")
        .add_model(fault_manager, fm_mbox, "FAULT_MANAGER"))
}

fn main() {
    server::run(simulation, "0.0.0.0:41633".parse().unwrap()).unwrap()
}
