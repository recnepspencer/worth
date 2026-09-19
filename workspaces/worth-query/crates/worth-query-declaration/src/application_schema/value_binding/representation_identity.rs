/// Stable descriptive identity for the unit carried by a value binding.
///
/// The identity grants no conversion or schema authority. Installation must
/// match it against the field contract that consumes the binding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationUnitIdentity(&'static str);

impl ApplicationUnitIdentity {
    pub const fn declared(identity: &'static str) -> Self {
        Self(identity)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Stable descriptive identity for the coordinate frame carried by a value
/// binding.
///
/// Frame relationships remain graph meaning. This identity describes only the
/// representation expected by the binding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationFrameIdentity(&'static str);

impl ApplicationFrameIdentity {
    pub const fn declared(identity: &'static str) -> Self {
        Self(identity)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}
