use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};

use super::atomic_replacement::{self, AppliedPulseSourceDelta, PulseSourceActionFailure};
use super::{PulseSourceDeltaDefinitionFailure, PulseSourceDeltaIdentity};

const STATUS_FIELD: &[u8] = b"field status";
const REVISION_FIELD: &[u8] = b"field revision";

#[derive(Debug)]
pub(crate) struct RevisionSchemaSourceDelta {
    bytes: Box<[u8]>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StatusSchemaRecoverySourceDelta {
    canonical: CanonicalPlatformPulse,
}

impl RevisionSchemaSourceDelta {
    pub(crate) fn from_checked_in(
        canonical: CanonicalPlatformPulse,
    ) -> Result<Self, PulseSourceDeltaDefinitionFailure> {
        let source = canonical.source_bytes();
        let offsets = source
            .windows(STATUS_FIELD.len())
            .enumerate()
            .filter_map(|(offset, candidate)| (candidate == STATUS_FIELD).then_some(offset))
            .collect::<Vec<_>>();
        let [offset] = offsets.as_slice() else {
            return Err(match offsets.len() {
                0 => PulseSourceDeltaDefinitionFailure::StatusFieldMissing,
                count => PulseSourceDeltaDefinitionFailure::StatusFieldAmbiguous(count),
            });
        };
        let mut bytes =
            Vec::with_capacity(source.len() - STATUS_FIELD.len() + REVISION_FIELD.len());
        bytes.extend_from_slice(&source[..*offset]);
        bytes.extend_from_slice(REVISION_FIELD);
        bytes.extend_from_slice(&source[*offset + STATUS_FIELD.len()..]);
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
        })
    }

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
        atomic_replacement::apply(
            installation,
            PulseSourceDeltaIdentity::RevisionSchema,
            &self.bytes,
        )
    }

    pub(crate) fn source_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl StatusSchemaRecoverySourceDelta {
    pub(crate) fn exact(canonical: CanonicalPlatformPulse) -> Self {
        Self { canonical }
    }

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
        atomic_replacement::apply(
            installation,
            PulseSourceDeltaIdentity::StatusSchemaRecovery,
            self.canonical.source_bytes(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RevisionSchemaSourceDelta, StatusSchemaRecoverySourceDelta, REVISION_FIELD, STATUS_FIELD,
    };
    use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};

    #[test]
    fn schema_deltas_change_only_the_selected_field_and_restore_exact_canonical_source() {
        let canonical = CanonicalPlatformPulse::checked_in();
        let revision =
            RevisionSchemaSourceDelta::from_checked_in(canonical).expect("revision schema delta");
        assert_eq!(count(revision.source_bytes(), STATUS_FIELD), 0);
        assert_eq!(count(revision.source_bytes(), REVISION_FIELD), 1);
        let mut installation =
            IsolatedPulseInstallation::install(canonical).expect("isolated installation");
        let revision_bytes = revision.source_bytes().to_vec();
        let revision_receipt = revision.apply(&installation).expect("revision schema edit");
        assert_eq!(
            std::fs::read(installation.entry_source()).expect("revision source"),
            revision_bytes
        );

        let recovery = StatusSchemaRecoverySourceDelta::exact(canonical);
        let recovery_receipt = recovery
            .apply(&installation)
            .expect("status schema recovery");
        assert_eq!(
            std::fs::read(installation.entry_source()).expect("recovered source"),
            canonical.source_bytes()
        );
        assert_eq!(revision_receipt.action_count(), 1);
        assert_eq!(recovery_receipt.action_count(), 1);
        installation.close().expect("explicit cleanup");
    }

    fn count(source: &[u8], token: &[u8]) -> usize {
        source
            .windows(token.len())
            .filter(|candidate| *candidate == token)
            .count()
    }
}
