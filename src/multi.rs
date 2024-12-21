use malachite_core_types::Round;

use crate::context::{height::BaseHeight, value::BaseValue};


pub trait MultiProposer {
    fn get_proposal_part(&self, h: BaseHeight, r: Round) -> BaseValue;
}
