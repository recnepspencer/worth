use super::super::*;

pub(super) fn select_staged_current(
    candidate: &super::super::publication_candidate::RecoveryCandidateBasis,
    store: StableStoreIdentity,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> Result<
    (
        worth_store_physical_format::DurableRootSelector,
        crate::entry::PhysicalRecoveryRootProtocolCounters,
    ),
    ExecutionBasisDenial,
> {
    if candidate.artifacts.is_empty() {
        return Ok((
            candidate.staged_current_selector,
            crate::entry::PhysicalRecoveryRootProtocolCounters::default(),
        ));
    }
    let staged = candidate
        .artifacts
        .iter()
        .find(|candidate| {
            matches!(
                candidate.artifact(),
                RecordArtifactFile::RootSelectorCandidate {
                    role: worth_store_physical_format::RootSelectorRole::Current,
                    ..
                }
            )
        })
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let publication = match staged.artifact() {
        RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Current,
            publication,
        } => publication,
        _ => return Err(ExecutionBasisDenial::Invalid),
    };
    let selector =
        crate::integrity_ingress::admit_staged_current_selector(staged.bytes(), store, format)
            .map_err(|rejection| ExecutionBasisDenial::RootProtocol {
                artifact:
                    crate::entry::PhysicalRecoveryRootProtocolArtifact::StagedCurrentSelector {
                        publication,
                    },
                denial: rejection.diagnostic(),
                counters: Default::default(),
            })?
            .project();
    Ok((
        selector,
        crate::entry::PhysicalRecoveryRootProtocolCounters::default()
            .with_staged_selector_closeout(),
    ))
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_physical_format::{
        DurablePhysicalRootManifest, DurableRootSelector, FreeSpaceBlockReference, FreeSpaceKey,
        PhysicalRecordFormatDeclaration, RecordAllocationClass, RecordArtifactFile,
        RootSelectorIdentity, RootSelectorRole,
    };

    use super::*;

    #[test]
    fn noncanonical_staged_selector_retains_typed_publication_denial() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([23; 16]).unwrap(),
        )
        .published_identity();
        let selector = DurableRootSelector::new(
            store,
            format,
            RootSelectorIdentity::new(1).unwrap(),
            RootSelectorRole::Current,
            1,
            None,
            None,
        )
        .unwrap();
        let mut bytes = selector.encode();
        bytes[22] = 1;
        let checksum = crc32c_parts(&[&bytes[..44], &bytes[48..]]);
        bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
        let artifact = RecoveryPublicationCandidateArtifact {
            artifact: RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Current,
                publication: 9,
            },
            bytes: bytes.to_vec().into_boxed_slice(),
            payload_digest: [0; 32],
        };
        let key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
        let free = FreeSpaceBlockReference::new(1, 1, 0, 17, key, key).unwrap();
        let candidate = super::super::publication_candidate::RecoveryCandidateBasis {
            root: DurablePhysicalRootManifest::builder(1, 7, 4, 19)
                .free_space_root(Some(free))
                .admit()
                .unwrap(),
            referenced_artifacts: Box::new([]),
            artifacts: Box::new([artifact]),
            materialization_cost: Default::default(),
            staged_current_selector: selector,
        };

        let Err(ExecutionBasisDenial::RootProtocol {
            artifact:
                crate::entry::PhysicalRecoveryRootProtocolArtifact::StagedCurrentSelector {
                    publication: 9,
                },
            denial:
                crate::entry::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
                        localization,
                    ),
                ),
            counters,
        }) = select_staged_current(&candidate, store, format)
        else {
            panic!("staged selector damage must retain its publication coordinate");
        };
        assert_eq!(
            localization.cause(),
            worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure
        );
        assert_eq!(
            localization.field(),
            Some(worth_store_physical_integrity::PhysicalFormatField::Reserved)
        );
        assert_eq!(localization.damaged_range().offset(), 22);
        assert_eq!(localization.damaged_range().length(), 2);
        assert_eq!(counters.staged_selector_integrity_admissions(), 0);
        assert_eq!(counters.closeout_selector_interpretations(), 0);
    }

    fn crc32c_parts(parts: &[&[u8]]) -> u32 {
        let mut value = !0_u32;
        for byte in parts.iter().flat_map(|part| part.iter()) {
            value ^= u32::from(*byte);
            for _ in 0..8 {
                let mask = 0_u32.wrapping_sub(value & 1);
                value = (value >> 1) ^ (0x82f6_3b78 & mask);
            }
        }
        !value
    }
}
