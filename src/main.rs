// The loopback example demonstrates the simplest way to instantiate
// the Malachite library.
//
// We will use a purely local instance of Malachite. The approach is to
// simulate everything, the network, signing, mempool, etc.
// Each peer is a simple data structure. Messages passing via direct
// function calls.
//
// TODO (This will take time) --
//  The experience of building a system on top of Malachite in this example
//  should be no different from building on top of an SQLite instance.

use crossbeam_channel::Sender;
use std::process::exit;
use std::sync::mpsc::Receiver;
use std::thread;
use tracing::level_filters::LevelFilter;
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

use crate::context::value::BaseValue;
use crate::decision::Decision;
use crate::simulator::Simulator;

mod application;
mod common;
mod context;
mod decision;
mod simulator;

fn main() {
    // Some sensible defaults to make logging work
    init();

    // Create a network of 4 peers
    let (mut n, mut states, proposals, decisions) = Simulator::new(4);

    // Spawn a thread that produces values to be proposed
    produce_proposals_background(proposals);

    // Spawn a thread in the background that handles decided values
    consume_decisions_background(decisions);

    // Blocking method, starts the network and handles orchestration of
    // block building
    n.run(&mut states);

    // Todo: Clean stop
}

fn produce_proposals_background(proposals: Sender<BaseValue>) {
    let mut counter = 45;
    thread::spawn(move || loop {
        proposals
            .send(BaseValue(counter))
            .expect("could not send new value to be proposed");
        warn!(value = %counter, "IN -> new value to be proposed");

        counter += 1;
    });
}

fn consume_decisions_background(rx: Receiver<Decision>) {
    thread::spawn(move || {
        // Busy loop, simply consume the decided heights
        loop {
            let res = rx.recv();
            match res {
                Ok(d) => {
                    warn!(
                        peer = %d.peer.to_string(),
                        value = %d.value_id.to_string(),
                        height = %d.height,
                        "OUT <- new decision took place",
                    );
                }
                Err(err) => {
                    error!(error = ?err, "error receiving decisions");
                    error!("stopping");
                    exit(0);
                }
            }
        }
    });
}

fn init() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .unwrap()
        .add_directive("malachite_simulator=trace".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .with_target(false)
        .init();
}
