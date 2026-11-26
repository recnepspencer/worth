//! An application's own structured reason for refusing a request.
//!
//! One bounded value names which business rule failed (a machine-readable
//! `code`) and what to tell the operator (a one-line `message`). Operation
//! handlers return it through `WorthQueryOperationBodyDenial::DomainRefused`
//! (or their own denial implementing [`WorthQueryDomainRefusalCarrier`]);
//! installed invariants return it with [`WorthQueryDomainRefusalBinding`] as
//! their `RefusalBinding`. Both reach the caller unchanged.
mod canonical_binding;
mod carrier;
mod grammar;
#[cfg(test)]
mod tests;

pub use canonical_binding::WorthQueryDomainRefusalBinding;
pub use carrier::WorthQueryDomainRefusalCarrier;
pub use grammar::{
    WorthQueryDomainRefusalDenial, WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES,
    WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES,
};

use serde::{Deserialize, Deserializer};
use worth_foundational::facade::WorthValue;

/// A business rule's refusal: a stable `code` and an operator-facing `message`.
///
/// The code is 1 to 64 bytes of `a-z` followed by `a-z`, `0-9`, `_`, `.` or
/// `-` (for example `lot_expired` or `stock.bin_capacity_exceeded`); branch on
/// it, never on the message. The message is 1 to 256 bytes of non-blank text
/// with no control characters, shown to the user verbatim. Both bounds are
/// checked before anything is allocated, so an oversized or malformed reason
/// is refused deterministically and never truncated.
#[derive(Clone, Debug, Eq, PartialEq, Hash, WorthValue)]
pub struct WorthQueryDomainRefusal {
    code: String,
    message: String,
}

impl WorthQueryDomainRefusal {
    /// Validates both parts, then copies them into one owned refusal.
    pub fn new(code: &str, message: &str) -> Result<Self, WorthQueryDomainRefusalDenial> {
        grammar::admit_code(code)?;
        grammar::admit_message(message)?;
        Ok(Self {
            code: code.to_owned(),
            message: message.to_owned(),
        })
    }

    /// The stable machine-readable identifier of the refused rule.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The one-line operator message, shown verbatim.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Re-checks a value that bypassed [`Self::new`], such as a decoded one.
    pub(crate) fn admit(&self) -> Result<(), WorthQueryDomainRefusalDenial> {
        grammar::admit_code(&self.code)?;
        grammar::admit_message(&self.message)
    }
}

/// Renders the operator message; log `{:?}` to keep the code as well.
impl std::fmt::Display for WorthQueryDomainRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

/// Decoding admits the same grammar as [`WorthQueryDomainRefusal::new`], so a
/// stored or transported denial can never carry an invalid refusal.
impl<'de> Deserialize<'de> for WorthQueryDomainRefusal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            code: String,
            message: String,
        }
        let wire = Wire::deserialize(deserializer)?;
        let refusal = Self {
            code: wire.code,
            message: wire.message,
        };
        refusal.admit().map_err(serde::de::Error::custom)?;
        Ok(refusal)
    }
}
