use core::fmt;

use malachite_core_types::Height;

/// Base implementation of a Height
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaseHeight(pub u64);

impl BaseHeight {
    #[allow(dead_code)]
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }
}

impl Default for BaseHeight {
    fn default() -> Self {
        Self(0)
    }
}

impl fmt::Display for BaseHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Height for BaseHeight {
    fn increment_by(&self, n: u64) -> Self {
        Self(self.0 + n)
    }

    fn decrement_by(&self, n: u64) -> Option<Self> {
        match self.0 >= n {
            true => Some(Self(self.0 - n)),
            false => None,
        }
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}
