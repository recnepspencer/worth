use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_physical_integrity::{
    PhysicalBlastRadius, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalIntegrityRejection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootProtocolAdmissionDenial {
    FixedSelectorSlotAbsent(PhysicalDamageLocalization),
    ConflictingFixedSelectorDuplication(PhysicalDamageLocalization),
    AddressedRootAbsent(PhysicalDamageLocalization),
    Validation(PhysicalIntegrityRejection),
    SourceArtifactMismatch,
    SourceRangeMismatch,
    SourceIncarnationMismatch,
    ResidentFrame,
    OwnerProjectionRejected,
}

impl RootProtocolAdmissionDenial {
    pub(in crate::physical_runtime) fn fixed_selector_absent(
        store: StableStoreIdentity,
        format: PhysicalRecordFormatDeclaration,
        artifact: RecordArtifactFile,
    ) -> Self {
        let range = worth_store_physical_integrity::PhysicalByteRange::new(
            0,
            worth_store_physical_format::ROOT_SELECTOR_BYTES as u64,
        )
        .expect("fixed selector range is nonzero");
        let scope = match artifact {
            RecordArtifactFile::CurrentRootSelector => {
                worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(
                    store, format, range,
                )
            }
            RecordArtifactFile::PreviousRootSelector => {
                worth_store_physical_integrity::PhysicalArtifactScope::previous_root_selector(
                    store, format, range,
                )
            }
            _ => return Self::SourceArtifactMismatch,
        };
        Self::FixedSelectorSlotAbsent(PhysicalDamageLocalization::new(
            scope,
            PhysicalDamageCause::MissingArtifact,
            range,
            None,
            PhysicalBlastRadius::ReachableSubtree,
        ))
    }

    pub(in crate::physical_runtime) fn addressed_root_absent(
        store: StableStoreIdentity,
        format: PhysicalRecordFormatDeclaration,
        generation: u64,
        length: u64,
    ) -> Self {
        let Ok(range) = worth_store_physical_integrity::PhysicalByteRange::new(0, length) else {
            return Self::SourceRangeMismatch;
        };
        let Ok(scope) = worth_store_physical_integrity::PhysicalArtifactScope::root_manifest(
            store, format, generation, range,
        ) else {
            return Self::SourceArtifactMismatch;
        };
        Self::AddressedRootAbsent(PhysicalDamageLocalization::new(
            scope,
            PhysicalDamageCause::MissingArtifact,
            range,
            None,
            PhysicalBlastRadius::ReachableSubtree,
        ))
    }

    pub(in crate::physical_runtime) const fn from_validation(
        rejection: PhysicalIntegrityRejection,
    ) -> Self {
        Self::Validation(rejection)
    }

    /// Classifies duplication only after a route owner has independently
    /// observed both conflicting fixed namespace entries.
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn conflicting_fixed_selector_duplication_from_join(
        scope: worth_store_physical_integrity::PhysicalArtifactScope,
    ) -> Self {
        let range = scope.byte_range();
        Self::ConflictingFixedSelectorDuplication(PhysicalDamageLocalization::new(
            scope,
            PhysicalDamageCause::DuplicateArtifact,
            range,
            Some(worth_store_physical_integrity::PhysicalFormatField::SelectorRole),
            PhysicalBlastRadius::ReachableSubtree,
        ))
    }

    pub const fn localization(self) -> Option<PhysicalDamageLocalization> {
        match self {
            Self::FixedSelectorSlotAbsent(localization)
            | Self::ConflictingFixedSelectorDuplication(localization)
            | Self::AddressedRootAbsent(localization) => Some(localization),
            Self::Validation(PhysicalIntegrityRejection::Damaged(localization)) => {
                Some(localization)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "recovery-runtime-owner")]
    use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_physical_format::PhysicalRecordFormatDeclaration;
    #[cfg(feature = "recovery-runtime-owner")]
    use worth_store_physical_format::RecordArtifactFile;
    #[cfg(feature = "recovery-runtime-owner")]
    use worth_store_physical_integrity::PhysicalFormatField;
    use worth_store_physical_integrity::{
        PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    };

    use super::RootProtocolAdmissionDenial;

    #[cfg(feature = "recovery-runtime-owner")]
    #[test]
    fn fixed_slot_absence_retains_family_and_reachable_blast_radius() {
        let denial = RootProtocolAdmissionDenial::fixed_selector_absent(
            store(),
            format(),
            RecordArtifactFile::PreviousRootSelector,
        );
        let localization = denial.localization().unwrap();
        assert!(matches!(
            denial,
            RootProtocolAdmissionDenial::FixedSelectorSlotAbsent(_)
        ));
        assert_eq!(
            localization.scope().artifact_family(),
            PhysicalIntegrityArtifactFamily::PreviousRootSelector,
        );
        assert_eq!(localization.cause(), PhysicalDamageCause::MissingArtifact);
        assert_eq!(
            localization.damaged_range(),
            PhysicalByteRange::new(0, worth_store_physical_format::ROOT_SELECTOR_BYTES as u64)
                .unwrap(),
        );
        assert_eq!(localization.field(), None);
        assert_eq!(
            localization.blast_radius(),
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        );
    }

    #[test]
    fn addressed_root_absence_retains_exact_generation() {
        let denial = RootProtocolAdmissionDenial::addressed_root_absent(store(), format(), 7, 368);
        let localization = denial.localization().unwrap();
        assert!(matches!(
            denial,
            RootProtocolAdmissionDenial::AddressedRootAbsent(_)
        ));
        assert_eq!(localization.scope().root_generation(), Some(7));
        assert_eq!(localization.cause(), PhysicalDamageCause::MissingArtifact);
        assert_eq!(
            localization.damaged_range(),
            PhysicalByteRange::new(0, 368).unwrap(),
        );
        assert_eq!(localization.field(), None);
        assert_eq!(
            localization.blast_radius(),
            PhysicalBlastRadius::ReachableSubtree
        );
    }

    #[cfg(feature = "recovery-runtime-owner")]
    #[test]
    fn duplication_join_localization_retains_exact_fixed_slot() {
        let range =
            PhysicalByteRange::new(0, worth_store_physical_format::ROOT_SELECTOR_BYTES as u64)
                .unwrap();
        let scope = worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(
            store(),
            format(),
            range,
        );
        let denial =
            RootProtocolAdmissionDenial::conflicting_fixed_selector_duplication_from_join(scope);
        let localization = denial.localization().unwrap();
        assert_eq!(localization.scope(), scope);
        assert_eq!(localization.cause(), PhysicalDamageCause::DuplicateArtifact);
        assert_eq!(localization.damaged_range(), range);
        assert_eq!(
            localization.field(),
            Some(PhysicalFormatField::SelectorRole)
        );
        assert_eq!(
            localization.blast_radius(),
            PhysicalBlastRadius::ReachableSubtree
        );
    }

    fn store() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([0x71; 16]).unwrap(),
        )
        .published_identity()
    }

    fn format() -> PhysicalRecordFormatDeclaration {
        PhysicalRecordFormatDeclaration::builder().admit().unwrap()
    }
}
