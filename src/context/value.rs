use std::fmt;

use bytes::Bytes;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub struct BaseValue(pub u64);

impl BaseValue {
    pub fn to_bytes(&self) -> bytes::Bytes {
        // Todo: There's likely a better way
        Bytes::from(self.0.to_string())
    }
}

// Todo: Is this overkill?
//  Seems necessary for fulfilling Vote::value().
#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub struct BaseValueId(pub u64);

impl malachite_core_types::Value for BaseValue {
    type Id = BaseValueId;

    fn id(&self) -> Self::Id {
        BaseValueId(self.0)
    }
}

impl fmt::Display for BaseValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
