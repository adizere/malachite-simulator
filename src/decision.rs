use crate::context::address::BasePeerAddress;
use crate::context::height::BaseHeight;
use crate::context::value::BaseValueId;

#[derive(Debug)]
pub struct Decision {
    pub peer: BasePeerAddress,
    pub value_id: BaseValueId,
    pub height: BaseHeight,
}
