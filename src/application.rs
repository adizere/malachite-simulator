use crossbeam_channel as cbc;
use std::sync::mpsc::Sender;
use tracing::{debug, info, span, trace, Level};

use malachite_core_consensus::{
    ConsensusMsg, Effect, Error, Input, Params, ProposedValue, Resumable, Resume,
    SignedConsensusMsg, State, ValueToPropose,
};
use malachite_core_types::{
    CommitCertificate, Height, Round, SignedMessage, SigningProvider, Timeout, TimeoutKind,
    Validator, Validity, ValueOrigin, VoteType,
};
use malachite_metrics::Metrics;

use crate::context::address::BasePeerAddress;
use crate::context::height::BaseHeight;
use crate::context::peer_set::BasePeerSet;
use crate::context::value::BaseValue;
use crate::context::BaseContext;
use crate::decision::Decision;

/// Represents an [`Input`] message that the application logic
/// at a certain peers sends to another peer, potentially to self.
pub struct Envelope {
    pub source: BasePeerAddress,
    pub destination: BasePeerAddress,
    pub payload: Input<BaseContext>,
}

/// An application is the deterministic state machine executing
/// at a specific peer.
///
/// It contains:
/// (1) a reference to a [`Sender`], which it uses
/// to transmit [`Input`]s to itself and other application instances
/// running at other peers.
///
/// (2) a reference to a [`Sender`] to
/// communicate to the outside environment (the system) each
/// [`Decision`] this local application took.
///
/// (3) reference to a [`Receiver`] that provides to the app
/// the values to propose in each new height.
///
/// The application is a wrapper over the malachite consensus state
/// machine. It calls `malachite::process!` and handles
/// [`Effect`]s produced by the consensus library.
pub struct Application {
    pub peer_id: BasePeerAddress,

    /// Send [`Input`]s to the application running at self and other peers.
    pub network_tx: Sender<Envelope>,

    // Send [`Decision`]s to the environment, i.e., the [`System`].
    pub decision_tx: Sender<Decision>,

    // Consume the possible values that will be proposed
    pub proposal_rx: cbc::Receiver<BaseValue>,
}

impl Application {
    pub fn init(&self, initial_validator_set: BasePeerSet) {
        let input = Input::StartHeight(BaseHeight(0), initial_validator_set);

        let envelope = Envelope {
            // Send this envelope to self
            destination: self.peer_id,
            source: self.peer_id,
            payload: input,
        };

        // This envelope will later be used in apply_input.
        self.network_tx.send(envelope).unwrap();
    }

    // Wrapper over `process!` macro to work around the confusion
    // in return types due to the loop { } inside that macro.
    pub fn apply_input(
        &self,
        input: Input<BaseContext>,
        peer_params: &Params<BaseContext>,
        metrics: &Metrics,
        peer_state: &mut State<BaseContext>,
        ctx: &BaseContext,
    ) -> Result<(), Error<BaseContext>> {
        malachite_core_consensus::process!(
            input: input,
            state: peer_state,
            metrics: metrics,
            with: effect =>
                self.handle_effect(peer_params, effect, ctx)
        )
    }

    fn handle_schedule_timeout(&self, t: Timeout) -> Result<Resume<BaseContext>, String> {
        let Timeout { round: _, kind } = t;

        // Special case to handle.
        // If it's a timeout for kind Commit, then handle this timeout instantly.
        // Signal to self that the timeout has elapsed.
        // This will prompt consensus to provide the effect `Decide` afterward.
        if kind == TimeoutKind::Commit {
            debug!("triggering TimeoutElapsed for Commit");

            self.network_tx
                .send(Envelope {
                    source: self.peer_id,
                    destination: self.peer_id,
                    payload: Input::TimeoutElapsed(t),
                })
                .unwrap();
        }

        Ok(Resume::Continue)
    }

    fn handle_publish(
        &self,
        v: SignedConsensusMsg<BaseContext>,
        peer_params: &Params<BaseContext>,
    ) -> Result<Resume<BaseContext>, String> {
        // Push the signed consensus message into the inbox of all peers
        // That's all that broadcast entails
        for destination in peer_params.initial_validator_set.peers.iter() {
            let destination_addr = destination.address();
            match v {
                SignedConsensusMsg::Vote(ref sv) => {
                    // Note: No need to broadcast the vote to self
                    if destination_addr != &self.peer_id {
                        self.network_tx
                            .send(Envelope {
                                source: self.peer_id,
                                destination: *destination_addr,
                                payload: Input::Vote(sv.clone()),
                            })
                            .unwrap()
                    }
                }
                SignedConsensusMsg::Proposal(ref sp) => {
                    self.network_tx
                        .send(Envelope {
                            source: self.peer_id,
                            destination: *destination_addr,
                            payload: Input::Proposal(sp.clone()),
                        })
                        .unwrap();

                    // Todo: This was not intuitive to find - source of confusion
                    self.network_tx
                        .send(Envelope {
                            source: self.peer_id,
                            destination: *destination_addr,
                            payload: Input::ProposedValue(
                                ProposedValue {
                                    height: sp.height,
                                    round: sp.round,
                                    valid_round: Round::Nil,
                                    proposer: sp.proposer.clone(),
                                    value: sp.value,
                                    validity: Validity::Valid,
                                    extension: None,
                                },
                                ValueOrigin::Consensus,
                            ),
                        })
                        .unwrap();
                }
            }
        }
        Ok(Resume::Continue)
    }

    fn handle_decide(
        &self,
        certificate: CommitCertificate<BaseContext>,
        peer_params: &Params<BaseContext>,
    ) -> Result<Resume<BaseContext>, String> {
        println!("decision arrived!");
        for s in certificate.aggregated_signature.signatures.iter() {
            println!("signature from {:?} with ext {:?}", s.address, s.extension);
        }

        // Let the top-level system/environment know about this decision
        self.decision_tx
            .send(Decision {
                peer: self.peer_id,
                value_id: certificate.value_id,
                height: certificate.height,
            })
            .expect("unable to send a decision");

        // Proceed to the next height
        let val_set = peer_params.initial_validator_set.clone();

        // Register the input in the inbox of this peer
        self.network_tx
            .send(Envelope {
                source: self.peer_id,
                destination: self.peer_id,
                payload: Input::StartHeight(certificate.height.increment(), val_set),
            })
            .unwrap();

        Ok(Resume::Continue)
    }

    // Control passes from consensus to the application here.
    // The app creates a value and provides it as input to Malachite
    // in the form of a `ValueToPropose` variant.
    // Register this input in the inbox of the current validator.
    // If no value is available, the application blocks waiting.
    fn handle_get_value(&self, h: BaseHeight, r: Round) -> Result<Resume<BaseContext>, String> {
        let value = self.proposal_rx.try_recv().or_else(|_| Err("no value"))?;

        let input_value = ValueToPropose {
            height: h,
            round: r,
            valid_round: Round::Nil,
            value,
            extension: None,
        };
        self.network_tx
            .send(Envelope {
                source: self.peer_id,
                destination: self.peer_id,
                payload: Input::Propose(input_value),
            })
            .unwrap();

        Ok(Resume::Continue)
    }

    fn handle_effect(
        &self,
        peer_params: &Params<BaseContext>,
        effect: Effect<BaseContext>,
        context: &BaseContext,
    ) -> Result<Resume<BaseContext>, String> {
        assert_eq!(peer_params.address, self.peer_id);

        let peer_id = peer_params.address;
        let span = span!(Level::INFO, "handle_effect for peer", "{}", peer_id.0);
        let _enter = span.enter();

        match effect {
            Effect::ResetTimeouts(c) => {
                trace!("ResetTimeouts");

                Ok(c.resume_with(()))
            }
            Effect::CancelAllTimeouts(c) => {
                trace!("CancelAllTimeouts");

                Ok(c.resume_with(()))
            }
            Effect::CancelTimeout(_, c) => {
                trace!("CancelTimeout");

                Ok(c.resume_with(()))
            }
            Effect::ScheduleTimeout(t, c) => {
                trace!("ScheduleTimeout {}", t);

                let _res = self.handle_schedule_timeout(t).unwrap();

                Ok(c.resume_with(()))
            }
            Effect::StartRound(_, _, _, c) => {
                trace!("StartRound");

                // Nothing in particular to keep track of

                Ok(c.resume_with(()))
            }
            Effect::Publish(v, c) => {
                info!("Publish {}", pretty_publish(&v));

                let _ = self.handle_publish(v, peer_params).unwrap();

                Ok(c.resume_with(()))
            }
            Effect::GetValue(h, r, _, c) => {
                trace!("GetValue");

                let _ = self.handle_get_value(h, r).unwrap();

                Ok(c.resume_with(()))
            }
            Effect::GetValidatorSet(h, c) => {
                info!("GetValidatorSet({}); providing the default", h);

                // Assumption: validator sets do not change.
                let val_set = peer_params.initial_validator_set.clone();

                Ok(c.resume_with(Some(val_set)))
            }
            Effect::VerifySignature(m, _, c) => {
                trace!("VerifySignature {}", pretty_verify_signature(m));

                // Consider implementing this to be able to capture more realistic
                // conditions.
                // Not required right now, given the current use of this application.

                Ok(c.resume_with(true))
            }
            Effect::Decide(certificate, c) => {
                trace!("Decide");

                // TODO: No need to return resume from here
                let _ = self.handle_decide(certificate, peer_params).unwrap();

                Ok(c.resume_with(()))
            }
            Effect::RestreamValue(_, _, _, _, _, _c) => {
                panic!("unimplemented arm Effect::RestreamValue in match effect");
            }
            Effect::PersistMessage(_, c) => {
                // No support for crash-recovery
                Ok(c.resume_with(()))
            }
            Effect::PersistTimeout(_, c) => {
                // No support for crash-recovery
                Ok(c.resume_with(()))
            }
            Effect::SignVote(v, c) => {
                let ext = if v.vote_type == VoteType::Precommit {
                    let part = self.get_proposal_part(v.height, v.round);
                    Some(part)
                } else {
                    None
                };
                let sv = context.signing_provider.sign_vote_extended(v, ext);

                Ok(c.resume_with(sv))
            }
            Effect::SignProposal(p, c) => {
                let sp = context.signing_provider.sign_proposal(p);

                Ok(c.resume_with(sp))
            }
            Effect::GetVoteSet(_, _, _) => {
                // Not needed
                // The app will never go into round > 0
                panic!("unimplemented arm Effect::GetVoteSet in match effect")
            }
            Effect::SendVoteSetResponse(_, _, _, _, _) => {
                // Not needed
                // The app will never go into round > 0
                panic!("unimplemented arm Effect::SendVoteSetResponse in match effect")
            }
            Effect::VerifyCertificate(_, _, _, _) => {
                panic!("unimplemented arm Effect::VerifyCertificate in match effect")
            }
        }
    }
}

fn pretty_publish(v: &SignedConsensusMsg<BaseContext>) -> String {
    match v {
        SignedConsensusMsg::Vote(ref sv) => sv.to_string(),
        SignedConsensusMsg::Proposal(ref sp) => sp.to_string(),
    }
}

fn pretty_verify_signature(m: SignedMessage<BaseContext, ConsensusMsg<BaseContext>>) -> String {
    match m.message {
        ConsensusMsg::Vote(v) => v.to_string(),
        ConsensusMsg::Proposal(p) => p.to_string(),
    }
}
