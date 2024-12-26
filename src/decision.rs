use crate::context::address::BasePeerAddress;
use crate::context::height::BaseHeight;
use crate::context::value::BaseValueId;

/// Represents the finalized value that a certain peer reached
/// for a certain height [`BaseHeight`].
///
/// The full value is not captured here, merely the
/// identifier of that value: [`BaseValueId`].
#[derive(Debug)]
pub struct Decision {
    pub peer: BasePeerAddress,
    pub value_id: BaseValueId,
    pub height: BaseHeight,
}
