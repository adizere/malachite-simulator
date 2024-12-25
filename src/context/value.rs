use std::fmt;

use bytes::Bytes;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub struct BaseValue(pub u64);

impl BaseValue {
    pub fn to_bytes(&self) -> Bytes {
        // Todo: There's likely a better way
        let r = self.0.to_be_bytes();
        Bytes::from(r.to_vec())
    }

    pub fn from_bytes(bytes: &Bytes) -> Self {
        // Todo: There's likely a better way
        let val: u64 = u64::from_be_bytes(bytes.to_vec().try_into().unwrap());
        BaseValue(val)
    }
}

impl fmt::Display for BaseValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
