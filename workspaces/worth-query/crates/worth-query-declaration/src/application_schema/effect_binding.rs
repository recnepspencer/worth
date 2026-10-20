use super::{ApplicationExternalEffectProtocol, ApplicationStructuredValueBinding};
mod projection;
pub use projection::ApplicationExternalEffectPayloadProjection;

/// A retained effect binding whose value may cross an external process boundary.
pub trait ApplicationExternalEffectBinding: ApplicationStructuredValueBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol;
    const MAX_EXTERNAL_BYTES: u64;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8>;
}
