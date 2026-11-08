//! Runtime slot ownership distinguishes authored claims from live binding keys.
use worth_foundational::facade::BridgeSemanticClaimReference;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum BridgeSemanticClaimOrigin {
    Memory(String),
    Recorded(BridgeSemanticClaimReference),
}
impl BridgeSemanticClaimOrigin {
    pub(crate) fn identity_basis(&self) -> String {
        match self {
            Self::Memory(key) => key.clone(),
            Self::Recorded(reference) => {
                use std::fmt::Write;
                let definition = reference.definition();
                let mut result = String::from("authored-claim:");
                for byte in definition.owner().home().bytes() {
                    write!(&mut result, "{byte:02x}")
                        .expect("writing to an owned string cannot fail");
                }
                write!(
                    &mut result,
                    ":{}:{}:{}",
                    definition.owner().slot().get(),
                    definition.ordinal(),
                    reference.ordinal()
                )
                .expect("writing to an owned string cannot fail");
                result
            }
        }
    }
}
