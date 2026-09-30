//! Installed external-effect contract for aftermath classification.

use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

/// Installed external-effect posture bound into an aftermath contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstalledExternalEffectContract {
    None,
    Declared {
        correlation_family: WorthQueryExternalEffectCorrelationFamily,
        effect: String,
        rust_payload_type: WorthQueryPortableTypeIdentity,
        protocol: ApplicationExternalEffectProtocol,
        maximum_payload_bytes: u64,
        inbound: Option<crate::application_schema::InstalledInboundOccurrenceContract>,
    },
}

impl InstalledExternalEffectContract {
    pub const fn is_declared(&self) -> bool {
        matches!(self, Self::Declared { .. })
    }

    /// The correlation family this operation escapes through, if any.
    pub const fn correlation_family(&self) -> Option<&WorthQueryExternalEffectCorrelationFamily> {
        match self {
            Self::Declared {
                correlation_family, ..
            } => Some(correlation_family),
            Self::None => None,
        }
    }

    pub fn effect(&self) -> Option<&str> {
        match self {
            Self::Declared { effect, .. } => Some(effect),
            Self::None => None,
        }
    }

    /// Stable payload protocol identity carried by a declared external lane.
    pub const fn protocol(&self) -> Option<&ApplicationExternalEffectProtocol> {
        match self {
            Self::Declared { protocol, .. } => Some(protocol),
            Self::None => None,
        }
    }

    pub const fn inbound(
        &self,
    ) -> Option<&crate::application_schema::InstalledInboundOccurrenceContract> {
        match self {
            Self::Declared { inbound, .. } => inbound.as_ref(),
            Self::None => None,
        }
    }
}

/// Classification-only external posture carried by the aftermath contract.
///
/// This is a *projection* of the operation's installed external-effect
/// contract, never an independent declaration. The operation contract owns the
/// typed payload projection and the correlation family; aftermath needs only to
/// know whether an external owner participates and through which family.
///
/// Deriving it is what makes the reversibility guard honest. While aftermath
/// declared its own posture, the guard asked the aftermath contract whether the
/// operation escaped, and an operation that declared the real lane through
/// an operation external-effect slot but omitted the aftermath posture installed as
/// `Reversible` — Query offered undo over an effect that had already left the
/// process (Q8.25-C1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstalledExternalEffectPosture {
    None,
    Declared {
        correlation_family: WorthQueryExternalEffectCorrelationFamily,
    },
}

impl InstalledExternalEffectPosture {
    pub(crate) fn from_operation_contract(contract: &InstalledExternalEffectContract) -> Self {
        match contract.correlation_family() {
            Some(correlation_family) => Self::Declared {
                correlation_family: correlation_family.clone(),
            },
            None => Self::None,
        }
    }

    pub const fn is_declared(&self) -> bool {
        matches!(self, Self::Declared { .. })
    }

    pub const fn correlation_family(&self) -> Option<&WorthQueryExternalEffectCorrelationFamily> {
        match self {
            Self::Declared { correlation_family } => Some(correlation_family),
            Self::None => None,
        }
    }
}
