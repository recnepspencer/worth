use super::{ApplicationExternalEffectProtocol, ApplicationStructuredValueBinding};

/// A structured effect binding that accounts for the memory retained by its value.
pub trait ApplicationRetainedEffectBinding: ApplicationStructuredValueBinding {
    fn retained_bytes(value: &Self::Value) -> u64;
}

/// A retained effect binding whose value may cross an external process boundary.
pub trait ApplicationExternalEffectBinding: ApplicationRetainedEffectBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol;
    const MAX_EXTERNAL_BYTES: u64;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8>;
}
