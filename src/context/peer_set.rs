/// Implementation of `PeerSet` and some utility methods.
///
use std::cmp::PartialEq;
use tracing::warn;

use crate::context::address::BasePeerAddress;
use crate::context::peer::BasePeer;
use crate::context::BaseContext;

use malachite_core_types::{ValidatorSet, VotingPower};

/// A minimal type capturing a set of peers.
/// Implements [`ValidatorSet`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasePeerSet {
    pub peers: Vec<BasePeer>,
}

impl BasePeerSet {
    /// Create a new set of peers of cardinality `size`.
    pub fn new(size: u32) -> Self {
        let mut peers = vec![];

        for i in 0..size {
            let peer = BasePeer::new(i);
            warn!(peer = %i, "created");

            peers.push(peer);
        }

        peers.into()
    }
}

impl From<Vec<BasePeer>> for BasePeerSet {
    fn from(value: Vec<BasePeer>) -> Self {
        Self { peers: value }
    }
}

impl ValidatorSet<BaseContext> for BasePeerSet {
    fn count(&self) -> usize {
        self.peers.len()
    }

    // Note: VotingPower is a primitive we can simply re-use
    fn total_voting_power(&self) -> VotingPower {
        // Todo: Double-check if this is fishy
        self.count() as u64
    }

    fn get_by_address(&self, address: &BasePeerAddress) -> Option<&BasePeer> {
        self.peers.iter().find(|v| &v.id == address)
    }

    fn get_by_index(&self, index: usize) -> Option<&BasePeer> {
        self.peers.get(index)
    }
}
