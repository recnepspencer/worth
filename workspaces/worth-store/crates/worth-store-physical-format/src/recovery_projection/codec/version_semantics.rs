//! Version-to-semantic-family contract for the canonical redo projection.
//! V13/V14 carry the family explicitly; older versions reserve four special
//! families for their version tags while ordinary append families remain valid.

use super::super::{
    PersistedPhysicalRecoveryBlobSemantic, PhysicalRecoveryProjectionDenial,
    RecoveryProjectionVersion,
};

pub(super) fn admit(
    version: RecoveryProjectionVersion,
    semantic: PersistedPhysicalRecoveryBlobSemantic,
) -> Result<(), PhysicalRecoveryProjectionDenial> {
    if matches!(
        version,
        RecoveryProjectionVersion::V13 | RecoveryProjectionVersion::V14
    ) {
        return Ok(());
    }
    let released = matches!(
        semantic,
        PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(_)
    );
    let derived = matches!(
        semantic,
        PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(_)
    );
    let reused = matches!(
        semantic,
        PersistedPhysicalRecoveryBlobSemantic::ChunkReused(_)
    );
    let quarantined = matches!(
        semantic,
        PersistedPhysicalRecoveryBlobSemantic::DedupeQuarantined(_)
    );
    if (version == RecoveryProjectionVersion::V7) != released
        || matches!(
            version,
            RecoveryProjectionVersion::V8
                | RecoveryProjectionVersion::V10
                | RecoveryProjectionVersion::V12
        ) != derived
        || (version == RecoveryProjectionVersion::V9) != reused
        || (version == RecoveryProjectionVersion::V11) != quarantined
    {
        Err(PhysicalRecoveryProjectionDenial::Malformed)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{PersistedBlobSemanticRecordBinding, PersistedRecordIdentity};

    use super::*;

    #[test]
    fn ordinary_family_remains_valid_while_special_tags_are_exclusive() {
        let record = PersistedRecordIdentity::new([1; 16], 2).unwrap();
        let binding = PersistedBlobSemanticRecordBinding::new(record, [2; 32], 3).unwrap();
        assert_eq!(
            admit(
                RecoveryProjectionVersion::V6,
                PersistedPhysicalRecoveryBlobSemantic::SessionDeclared(binding)
            ),
            Ok(())
        );
        assert_eq!(
            admit(
                RecoveryProjectionVersion::V7,
                PersistedPhysicalRecoveryBlobSemantic::SessionDeclared(binding)
            ),
            Err(PhysicalRecoveryProjectionDenial::Malformed)
        );
        assert_eq!(
            admit(
                RecoveryProjectionVersion::V7,
                PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
            ),
            Ok(())
        );
        assert_eq!(
            admit(
                RecoveryProjectionVersion::V6,
                PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
            ),
            Err(PhysicalRecoveryProjectionDenial::Malformed)
        );
        assert_eq!(
            admit(
                RecoveryProjectionVersion::V14,
                PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
            ),
            Ok(())
        );
    }
}
