use malachite_core_types::{Context, NilOrVal, Round, ValueId, VoteType};

use address::BasePeerAddress;
use height::BaseHeight;
use peer::BasePeer;
use peer_set::BasePeerSet;
use proposals::{BaseProposal, BaseProposalPart};
use signing_provider::BaseSigningProvider;
use value::BaseValue;
use vote::BaseVote;

// Type definitions needed for the context
pub mod address;
pub mod height;
pub mod peer;
pub mod peer_set;
pub mod proposals;
pub mod signing_provider;
pub mod signing_scheme;
pub mod value;
pub mod vote;

#[derive(Clone)]
pub struct BaseContext {
    pub signing_provider: BaseSigningProvider,
}

impl BaseContext {
    pub fn new() -> BaseContext {
        BaseContext {
            signing_provider: BaseSigningProvider::new(),
        }
    }
}

#[allow(unused_variables)]
impl Context for BaseContext {
    type Address = BasePeerAddress;
    type Height = BaseHeight;
    type ProposalPart = BaseProposalPart;
    type Proposal = BaseProposal;
    type Validator = BasePeer;
    type ValidatorSet = BasePeerSet;
    type Value = BaseValue;
    type Vote = BaseVote;
    type SigningProvider = BaseSigningProvider;
    type SigningScheme = signing_scheme::Ed25519;

    fn select_proposer<'a>(
        &self,
        validator_set: &'a Self::ValidatorSet,
        height: Self::Height,
        round: Round,
    ) -> &'a Self::Validator {
        // Keep it simple, the proposer is always the same peer
        validator_set
            .peers
            .get(0)
            .expect("no peer found in the validator set")
    }

    fn new_proposal(
        height: Self::Height,
        round: Round,
        value: Self::Value,
        pol_round: Round,
        address: Self::Address,
    ) -> Self::Proposal {
        BaseProposal {
            height,
            value,
            proposer: address,
            round,
        }
    }

    fn new_prevote(
        height: Self::Height,
        round: Round,
        value_id: NilOrVal<ValueId<Self>>,
        address: Self::Address,
    ) -> Self::Vote {
        BaseVote {
            vote_type: VoteType::Prevote,
            height,
            value_id,
            round,
            voter: address,
            // TODO: A bit strange there is option to put extension into Prevotes
            //  clarify.
            extension: None,
        }
    }

    fn new_precommit(
        height: Self::Height,
        round: Round,
        value_id: NilOrVal<ValueId<Self>>,
        address: Self::Address,
    ) -> Self::Vote {
        BaseVote {
            vote_type: VoteType::Precommit,
            height,
            value_id,
            round,
            voter: address,
            extension: None,
        }
    }

    fn signing_provider(&self) -> &Self::SigningProvider {
        &self.signing_provider
    }
}
