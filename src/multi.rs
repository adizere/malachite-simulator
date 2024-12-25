use malachite_core_types::{CommitCertificate, Round};

use crate::context::{height::BaseHeight, value::BaseValue, BaseContext};

/// A block created by multiple proposers.
/// The block simply consists of a vector of values, and has the height attached to it.
/// Each value originated from a specific proposer.
pub struct MultiPropBlock {
    pub values: Vec<BaseValue>,
    pub height: BaseHeight,
}

pub trait MultiProposer {
    fn get_single_prop(&self, h: BaseHeight, r: Round) -> BaseValue;
    fn handle_multi_prop(&self, p0: &CommitCertificate<BaseContext>);
    // fn validate_multi_prop(&self)
}
