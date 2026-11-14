//! Portable denial DATA keeps the original category and scalar bytes.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorthQueryRecordedProducerDenialValue {
    Undeclared,
    Absent {
        identity: String,
    },
    Value {
        identity: String,
        bytes: Vec<u8>,
    },
    EncodingRefused {
        identity: String,
        cause: WorthQueryRecordedProducerDenialEncodingRefusal,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorthQueryRecordedProducerDenialEncodingRefusal {
    Codec { description: String },
    NoncanonicalCarrier,
    NoFiniteWireBound,
    ExceedsWireBound { maximum: usize, observed: usize },
}
impl WorthQueryRecordedProducerDenialValue {
    pub(in crate::domain_computation::primary_graph) fn capture(
        value: &WorthQueryProducerDenialValue,
    ) -> Self {
        match value {
            WorthQueryProducerDenialValue::Recorded(value) => value.as_ref().clone(),
            WorthQueryProducerDenialValue::Undeclared => Self::Undeclared,
            WorthQueryProducerDenialValue::Absent { value_identity } => Self::Absent {
                identity: (*value_identity).to_owned(),
            },
            WorthQueryProducerDenialValue::DeclaredValue {
                value_identity,
                bytes,
            } => Self::Value {
                identity: (*value_identity).to_owned(),
                bytes: bytes.clone(),
            },
            WorthQueryProducerDenialValue::EncodingRefused {
                value_identity,
                cause,
            } => Self::EncodingRefused {
                identity: (*value_identity).to_owned(),
                cause: match cause {
                    WorthQueryProducerDenialEncodingRefusal::Codec(cause) => {
                        WorthQueryRecordedProducerDenialEncodingRefusal::Codec {
                            description: format!("{cause:?}"),
                        }
                    }
                    WorthQueryProducerDenialEncodingRefusal::NoncanonicalCarrier => {
                        WorthQueryRecordedProducerDenialEncodingRefusal::NoncanonicalCarrier
                    }
                    WorthQueryProducerDenialEncodingRefusal::NoFiniteWireBound => {
                        WorthQueryRecordedProducerDenialEncodingRefusal::NoFiniteWireBound
                    }
                    WorthQueryProducerDenialEncodingRefusal::ExceedsWireBound {
                        maximum,
                        observed,
                    } => WorthQueryRecordedProducerDenialEncodingRefusal::ExceedsWireBound {
                        maximum: *maximum,
                        observed: *observed,
                    },
                },
            },
        }
    }
    pub fn identity(&self) -> Option<&str> {
        match self {
            Self::Undeclared => None,
            Self::Absent { identity }
            | Self::Value { identity, .. }
            | Self::EncodingRefused { identity, .. } => Some(identity),
        }
    }
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Value { bytes, .. } => Some(bytes),
            _ => None,
        }
    }
    pub(super) fn decode<B: ApplicationReadableScalarValueBinding>(
        &self,
    ) -> Result<B::Value, WorthQueryProducerDenialDecodeDenial> {
        use WorthQueryProducerDenialDecodeDenial as D;
        let Some(identity) = self.identity() else {
            return Err(D::Undeclared);
        };
        if identity != B::IDENTITY_NAME {
            return Err(D::RecordedTypeMismatch {
                expected: B::IDENTITY_NAME,
                observed: identity.to_owned(),
            });
        }
        let bytes = match self {
            Self::Undeclared => return Err(D::Undeclared),
            Self::Absent { .. } => return Err(D::Absent),
            Self::EncodingRefused { cause, .. } => {
                return Err(D::RecordedEncodingRefused(cause.clone()))
            }
            Self::Value { bytes, .. } => bytes,
        };
        if bytes.len() > canonical_wire_maximum::<B>().ok_or(D::InvalidEncoding)? {
            return Err(D::InvalidEncoding);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| D::InvalidEncoding)?;
        B::decode(&AspectValue::String(text.to_owned().into())).map_err(D::Value)
    }
    pub(super) fn owned_capacity_bytes(&self) -> u64 {
        match self {
            Self::Undeclared => 0,
            Self::Absent { identity } => identity.capacity() as u64,
            Self::Value { identity, bytes } => {
                (identity.capacity() as u64).saturating_add(bytes.capacity() as u64)
            }
            Self::EncodingRefused { identity, cause } => (identity.capacity() as u64)
                .saturating_add(match cause {
                    WorthQueryRecordedProducerDenialEncodingRefusal::Codec { description } => {
                        description.capacity() as u64
                    }
                    _ => 0,
                }),
        }
    }
}
