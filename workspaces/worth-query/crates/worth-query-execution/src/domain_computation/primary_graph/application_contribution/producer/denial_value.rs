//! A producer's typed domain denial crosses the demand boundary as a declared
//! canonical value. The declaring binding is the sole encoder and decoder.

use crate::domain_computation::primary_graph::application_invariant::refusal::canonical_wire_maximum;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial,
};

mod declaration;
pub use declaration::{
    WorthQueryProducerDenialDeclaration, WorthQueryProducerDenialProjection,
    WorthQueryProjectedProducerDenial, WorthQueryUndeclaredProducerDenial,
};

/// The domain value a rejected producer decision carries to its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerDenialValue {
    /// The producer explicitly declares no domain value for its denials.
    Undeclared,
    /// Canonical bytes from the declared binding; only that binding decodes them.
    DeclaredValue {
        value_identity: &'static str,
        bytes: Vec<u8>,
    },
    /// The declared projection gives this denial no value.
    Absent { value_identity: &'static str },
    /// The declared binding could not carry this denial. The domain decision
    /// still refused; its value is reported lost, never replaced or dropped.
    EncodingRefused {
        value_identity: &'static str,
        cause: WorthQueryProducerDenialEncodingRefusal,
    },
}

/// Why the declared binding could not encode a producer's domain denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerDenialEncodingRefusal {
    Codec(ApplicationValueEncodeDenial),
    NoncanonicalCarrier,
    NoFiniteWireBound,
    ExceedsWireBound { maximum: usize, observed: usize },
}

/// Why a caller's binding cannot read the demand's producer denial value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerDenialDecodeDenial {
    /// The demand was not refused by a producer's domain decision.
    NotDomainDenied,
    /// The producer declares no domain value for its denials.
    Undeclared,
    /// The declared projection gives this denial no value.
    Absent,
    /// The declared binding could not encode the original denial.
    EncodingRefused(WorthQueryProducerDenialEncodingRefusal),
    /// The caller's binding is not the producer's declared binding.
    TypeMismatch {
        expected: &'static str,
        observed: &'static str,
    },
    InvalidEncoding,
    Value(ApplicationValueDecodeDenial),
}

impl WorthQueryProducerDenialValue {
    /// Decode with the producer's declared binding. A different binding is
    /// refused before its decoder runs.
    pub fn decode<Binding: ApplicationReadableScalarValueBinding>(
        &self,
    ) -> Result<Binding::Value, WorthQueryProducerDenialDecodeDenial> {
        use WorthQueryProducerDenialDecodeDenial as Denial;
        let expected = Binding::IDENTITY_NAME;
        let (observed, bytes) = match self {
            Self::Undeclared => return Err(Denial::Undeclared),
            Self::Absent { value_identity } => {
                return Err(mismatch_or(expected, value_identity, Denial::Absent))
            }
            Self::EncodingRefused {
                value_identity,
                cause,
            } => {
                return Err(mismatch_or(
                    expected,
                    value_identity,
                    Denial::EncodingRefused(cause.clone()),
                ))
            }
            Self::DeclaredValue {
                value_identity,
                bytes,
            } => (*value_identity, bytes),
        };
        if observed != expected {
            return Err(Denial::TypeMismatch { expected, observed });
        }
        let maximum = canonical_wire_maximum::<Binding>().ok_or(Denial::InvalidEncoding)?;
        if bytes.len() > maximum {
            return Err(Denial::InvalidEncoding);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| Denial::InvalidEncoding)?;
        Binding::decode(&AspectValue::String(text.to_owned().into())).map_err(Denial::Value)
    }

    /// The exact canonical bytes the declared binding encoded.
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::DeclaredValue { bytes, .. } => Some(bytes),
            Self::Undeclared | Self::Absent { .. } | Self::EncodingRefused { .. } => None,
        }
    }

    /// The declared binding's identity, when the producer declares one.
    pub const fn value_identity(&self) -> Option<&'static str> {
        match self {
            Self::Undeclared => None,
            Self::DeclaredValue { value_identity, .. }
            | Self::Absent { value_identity }
            | Self::EncodingRefused { value_identity, .. } => Some(value_identity),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn owned_capacity_bytes(&self) -> u64 {
        match self {
            Self::DeclaredValue { bytes, .. } => bytes.capacity() as u64,
            Self::Undeclared | Self::Absent { .. } | Self::EncodingRefused { .. } => 0,
        }
    }
}

/// A recorded identity other than the caller's binding is a mismatch first.
fn mismatch_or(
    expected: &'static str,
    observed: &'static str,
    denial: WorthQueryProducerDenialDecodeDenial,
) -> WorthQueryProducerDenialDecodeDenial {
    if observed == expected {
        denial
    } else {
        WorthQueryProducerDenialDecodeDenial::TypeMismatch { expected, observed }
    }
}

#[cfg(test)]
mod tests;
