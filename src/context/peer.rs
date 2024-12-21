use rand::rngs::OsRng;
use std::fmt;

use malachite_core_types::{PublicKey, Validator, VotingPower};
use tracing::warn;

use crate::context::address::BasePeerAddress;
use crate::context::signing_scheme::{Ed25519, PrivateKey};
use crate::context::BaseContext;

/// This is the voting power of each peer.
pub const BASE_VOTING_POWER: u64 = 1;

/// The most basic definition of a peer.
/// All peers have equal voting power, [`BASE_VOTING_POWER`].
/// Implements [`Validator`] trait.
#[derive(Clone, Debug)]
pub struct BasePeer {
    pub id: BasePeerAddress,
    pub public_key: PublicKey<BaseContext>,
    private_key: PrivateKey,
}

impl BasePeer {
    pub fn new(id: u32) -> BasePeer {
        let csprng = OsRng;
        let signing_key = Ed25519::generate_keypair(csprng);

        warn!(verifying_key = ?signing_key.public_key(), "created new peer");

        BasePeer {
            id: BasePeerAddress::new(id),
            public_key: signing_key.public_key(),
            private_key: signing_key,
        }
    }
}

impl fmt::Display for BasePeer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "peer {}", self.id)
    }
}

impl PartialEq for BasePeer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BasePeer {}

impl Validator<BaseContext> for BasePeer {
    fn address(&self) -> &BasePeerAddress {
        &self.id
    }

    fn public_key(&self) -> &PublicKey<BaseContext> {
        &self.public_key
    }

    fn voting_power(&self) -> VotingPower {
        BASE_VOTING_POWER
    }
}
