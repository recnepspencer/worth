//! Recovered components retain only the owners that actually acted.
use std::sync::Arc;
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::{
    ReadmittedRelationalAssociatedEffect, RelationalRecoveredPublicationCustody,
};
use worth_signal::facade::durability::SignalReadmittedSourcePoint;

#[derive(Debug)]
pub(crate) enum RecoveredWorldOwnerEvidence {
    RelationalOnly(RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>),
    SignalOnly(Arc<SignalReadmittedSourcePoint>),
    RelationalAndSignal {
        relational: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
        signal: Arc<SignalReadmittedSourcePoint>,
    },
}
impl RecoveredWorldOwnerEvidence {
    pub(crate) fn from_admitted_components(
        relational: Option<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>,
        signal: Option<SignalReadmittedSourcePoint>,
    ) -> Option<Self> {
        Self::from_original_components(relational, signal.map(Arc::new))
    }
    pub(crate) fn from_original_components(
        relational: Option<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>,
        signal: Option<Arc<SignalReadmittedSourcePoint>>,
    ) -> Option<Self> {
        match (relational, signal) {
            (Some(relational), None) => Some(Self::RelationalOnly(relational)),
            (None, Some(signal)) => Some(Self::SignalOnly(signal)),
            (Some(relational), Some(signal)) => {
                Some(Self::RelationalAndSignal { relational, signal })
            }
            (None, None) => None,
        }
    }
    pub(crate) fn borrowed(&self) -> RecoveredWorldOwnerEvidenceRef<'_> {
        match self {
            Self::RelationalOnly(native) => {
                RecoveredWorldOwnerEvidenceRef::RelationalOnly(native.custody())
            }
            Self::SignalOnly(signal) => RecoveredWorldOwnerEvidenceRef::SignalOnly(signal),
            Self::RelationalAndSignal { relational, signal } => {
                RecoveredWorldOwnerEvidenceRef::RelationalAndSignal {
                    relational: relational.custody(),
                    signal,
                }
            }
        }
    }
    pub(super) fn into_meaning(self) -> ReadmittedOwnerMeaning {
        let native =
            |value: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>| {
                Arc::new(NativePublicationMeaning::Recovered(
                    value.consume(|value| value),
                ))
            };
        match self {
            Self::RelationalOnly(value) => ReadmittedOwnerMeaning::RelationalOnly(native(value)),
            Self::SignalOnly(value) => ReadmittedOwnerMeaning::SignalOnly(value),
            Self::RelationalAndSignal { relational, signal } => {
                ReadmittedOwnerMeaning::RelationalAndSignal {
                    relational: native(relational),
                    signal,
                }
            }
        }
    }
}
/// Borrowing preserves the original linear custody while constructing its successor.
#[derive(Clone, Copy)]
pub(crate) enum RecoveredWorldOwnerEvidenceRef<'a> {
    RelationalOnly(&'a RelationalRecoveredPublicationCustody),
    SignalOnly(&'a SignalReadmittedSourcePoint),
    RelationalAndSignal {
        relational: &'a RelationalRecoveredPublicationCustody,
        signal: &'a SignalReadmittedSourcePoint,
    },
}
impl<'a> RecoveredWorldOwnerEvidenceRef<'a> {
    pub(crate) fn relational(self) -> Option<&'a RelationalRecoveredPublicationCustody> {
        match self {
            Self::RelationalOnly(native)
            | Self::RelationalAndSignal {
                relational: native, ..
            } => Some(native),
            Self::SignalOnly(_) => None,
        }
    }
    pub(crate) fn signal(self) -> Option<&'a SignalReadmittedSourcePoint> {
        match self {
            Self::RelationalOnly(_) => None,
            Self::SignalOnly(signal) | Self::RelationalAndSignal { signal, .. } => Some(signal),
        }
    }
}
#[derive(Debug)]
pub(super) enum NativePublicationMeaning {
    Recovered(RelationalRecoveredPublicationCustody),
    Source(ReadmittedRelationalAssociatedEffect),
}
impl NativePublicationMeaning {
    pub(super) fn effect(&self) -> &ReadmittedRelationalAssociatedEffect {
        match self {
            Self::Recovered(value) => value.effect(),
            Self::Source(value) => value,
        }
    }
}
#[derive(Debug)]
pub(super) enum ReadmittedOwnerMeaning {
    RelationalOnly(Arc<NativePublicationMeaning>),
    SignalOnly(Arc<SignalReadmittedSourcePoint>),
    RelationalAndSignal {
        relational: Arc<NativePublicationMeaning>,
        signal: Arc<SignalReadmittedSourcePoint>,
    },
}
impl ReadmittedOwnerMeaning {
    pub(super) fn native(&self) -> Option<&Arc<NativePublicationMeaning>> {
        match self {
            Self::RelationalOnly(value)
            | Self::RelationalAndSignal {
                relational: value, ..
            } => Some(value),
            Self::SignalOnly(_) => None,
        }
    }
}
