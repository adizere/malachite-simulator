use crossbeam_channel as cbc;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, span, trace, Level};

use malachite_core_consensus::{Error, Input, Params, State, ValuePayload};
use malachite_metrics::Metrics;

use crate::application::{Application, Envelope};
use crate::common;
use crate::context::address::BasePeerAddress;
use crate::context::height::BaseHeight;
use crate::context::peer_set::BasePeerSet;
use crate::context::value::BaseValue;
use crate::context::BaseContext;
use crate::decision::Decision;

/// The delay between each consecutive step the simulator takes.
pub const STEP_DELAY: Duration = Duration::from_millis(200);

/// A system simulator represents:
///
/// - Some state of peers, namely params, metrics, application logic.
/// - The environment for executing the application and producing decisions: the network
///     layer which is simulated.
///
/// Upon triggering `run`, the simulator will:
///
/// - Pick arbitrary peers,
/// - Execute the peer's local state, e.g., send consensus messages, verify signatures.
/// - Eventually produce decisions that are streamed to outside the system through a `Receiver`.
pub struct Simulator {
    /// Params of each peer.
    params: HashMap<BasePeerAddress, Params<BaseContext>>,

    /// The metrics of each peer.
    metrics: HashMap<BasePeerAddress, Metrics>,

    // The application logic associated with each peer
    apps: HashMap<BasePeerAddress, Application>,

    // Simulates the networking layer, receiver-side
    // The sender-side is registered in all applications
    network_rx: Receiver<Envelope>,
}

impl Simulator {
    /// Creates a new system simulator consisting of `size` number of peers.
    /// Each peer is a validator in the system.
    ///
    /// Assumes the size of the system is >= 4 and < 25.
    pub fn new(
        size: u32,
    ) -> (
        Simulator,
        Vec<State<BaseContext>>,
        cbc::Sender<BaseValue>, // Proposals (input to the system)
        Receiver<Decision>,     // Decisions (output of the system)
    ) {
        assert!(size >= 4);
        assert!(size < 25);

        // Construct the simulated network
        let (ntx, nrx) = mpsc::channel();

        // Crossbeam channel on which `BaseValue` proposals pass from the environment into
        // application logic.
        // This is the mempool would be in a real application.
        let (ps, pr) = cbc::bounded(5);

        // Channel on which to send/receive the decisions
        let (dtx, drx) = mpsc::channel();

        let mut states = vec![];
        let mut params = HashMap::new();
        let mut apps = HashMap::new();

        // Construct the set of peers that comprise the network
        let ctx = BaseContext::new();
        let val_set = BasePeerSet::new(size, ctx.shared_public_key());

        // Construct the consensus states and params for each peer
        for i in 0..size {
            let peer_addr = BasePeerAddress::new(i);
            let p = Params {
                initial_height: BaseHeight::default(),
                initial_validator_set: val_set.clone(),
                address: peer_addr,
                // Note: The library provides a type and implementation
                // for threshold params which we're re-using.
                threshold_params: Default::default(),
                // Todo: This can be tricky, must be documented properly
                value_payload: ValuePayload::ProposalOnly,
            };

            // The params at this specific peer
            params.insert(peer_addr, p.clone());

            // The state at this specific peer
            let s = State::new(ctx.clone(), p);
            states.push(s);

            // Register the application corresponding to this peer
            apps.insert(
                peer_addr,
                Application {
                    peer_id: peer_addr,
                    network_tx: ntx.clone(),
                    decision_tx: dtx.clone(),
                    proposal_rx: pr.clone(),
                },
            );
        }

        (
            Simulator {
                params,
                metrics: HashMap::new(), // Initialize later, at `bootstrap` time
                apps,
                network_rx: nrx,
            },
            states,
            ps,
            drx,
        )
    }

    /// Orchestrate the execution of this system across the network of all peers.
    /// Running this will start producing decisions.
    pub fn run(&mut self, states: &mut Vec<State<BaseContext>>) {
        self.initialize_system(states);

        // Busy loop to orchestrate among peers
        loop {
            // Pick the next envelope from the network and demultiplex it
            self.step(states);

            // Simulate network and execution delays
            thread::sleep(STEP_DELAY);
        }
    }

    fn initialize_system(&mut self, states: &mut Vec<State<BaseContext>>) {
        let span = span!(Level::INFO, "initialize_system");
        let _enter = span.enter();

        for (_, peer_state) in states.iter_mut().enumerate() {
            let peer_addr = peer_state.params.address.clone();

            // Potentially a future refactor: Remove the self.params and
            // use the ones from peer_states instead.
            let peer_params = self
                .params
                .get(&peer_addr)
                .expect("could not identify peer at next position")
                .clone();

            // Initialize & save the metrics for later use
            let metrics = common::new_metrics();
            self.metrics.insert(peer_addr, metrics);

            // Tell the application at this peer to initialize itself
            let app = self.apps.get(&peer_addr).expect("app not found");
            app.init(peer_params.initial_validator_set.clone());

            debug!(peer = %peer_addr, "peer init done");
        }

        info!("done");
    }

    // Demultiplex among the incoming networking envelopes and call the corresponding
    // to handle the `Input`
    fn step(&mut self, states: &mut Vec<State<BaseContext>>) {
        let network_env = self.network_rx.recv();
        match network_env {
            Ok(envelope) => self.step_with_envelope(states, envelope),
            Err(err) => {
                error!(error = ?err, "error receiving the next envelope from the network");
            }
        }
    }

    fn step_with_envelope(&self, states: &mut Vec<State<BaseContext>>, envelope: Envelope) {
        let peer_addr = envelope.destination;

        let peer_state = states.get_mut(peer_addr.0 as usize).unwrap();
        let params = self.params.get(&peer_addr).unwrap().clone();
        let metrics = self.metrics.get(&peer_addr).unwrap().clone();
        let application = self.apps.get(&peer_addr).unwrap();

        let context = peer_state.ctx.clone();

        trace!(source = %envelope.source, destination = %envelope.destination, "applying an input from an envelope");

        self.apply_step_with_envelope(
            application,
            envelope.payload,
            &params,
            &metrics,
            peer_state,
            &context,
        )
        .expect("unknown error during process_peer");
    }

    fn apply_step_with_envelope(
        &self,
        application: &Application,
        input: Input<BaseContext>,
        peer_params: &Params<BaseContext>,
        metrics: &Metrics,
        peer_state: &mut State<BaseContext>,
        context: &BaseContext,
    ) -> Result<(), Error<BaseContext>> {
        application.apply_input(input, peer_params, metrics, peer_state, context)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::mpsc::TryRecvError;

    use crate::context::value::BaseValue;
    use crate::simulator::Simulator;

    #[test]
    fn basic_proposal_decisions() {
        const PEER_SET_SIZE: u32 = 4;

        let (mut n, mut states, proposals, decisions) = Simulator::new(PEER_SET_SIZE);
        n.initialize_system(&mut states);
        let mut proposal = 45;
        let mut peer_count = 0;

        for _i in 0..10 {
            // Create a value to be proposed
            proposals
                .send(BaseValue(proposal))
                .expect("could not send value to be proposed");

            println!("sent proposal {}", proposal);

            loop {
                // Let the system simulator take another step
                n.step(&mut states);

                // Check if the system reached a decision
                match decisions.try_recv() {
                    Ok(v) => {
                        println!("found decision {} from peer {}", v.value_id, v.peer);
                        let current_decision = v.value_id.0;
                        assert_eq!(current_decision, proposal);
                        peer_count += 1;

                        if peer_count == PEER_SET_SIZE {
                            // If all peers reached decision was reached, quit the inner loop
                            break;
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        // Keep trying, there was no decision yet
                    }
                    Err(_) => panic!("disconnected channel with decisions"),
                }
            }

            proposal += 1;
            peer_count = 0;
        }
    }
}
