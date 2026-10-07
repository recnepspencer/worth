//! Source-selected directory bytes joined to the second record of one
//! C.9-admitted released-drop member. The persisted effect is not proof.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryBinding,
    DerivedFamilyRootDirectoryV1, DurablePhysicalRootManifest, IndexedThroughBlobPublication,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryProjection,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
};

use crate::{AdmittedRootStepMemberView, ImmutablePhysicalRedoPlan, PhysicalRedoProjection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleasedDirectoryReplacementDenial {
    NotAdmittedReplacement,
    SourceBinding,
    SourcePayload,
    NewPayload,
}

/// Fixed-size proof; neither old nor new directory bytes are retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedReleasedDirectoryReplacement {
    operation: [u8; 32],
    source_root: DurablePhysicalRootManifest,
    result_root: Option<DurablePhysicalRootManifest>,
    candidate_generation: u64,
    expected_result_latest: Option<IndexedThroughBlobPublication>,
    previous: DerivedFamilyRootDirectoryBinding,
    previous_payload_sha256: [u8; 32],
    source_route: CurrentPhysicalRecordPlacement,
    next: DerivedFamilyRootDirectoryBinding,
    next_payload_sha256: [u8; 32],
    next_route: CurrentPhysicalRecordPlacement,
    descriptor_record: PersistedRecordIdentity,
}

impl VerifiedReleasedDirectoryReplacement {
    /// Peak transient directory decode storage for either admission or Store's
    /// independent media recheck; no frame-byte copy is retained.
    pub const fn maximum_decode_heap_bytes() -> u64 {
        (2 * worth_store_physical_format::MAX_DERIVED_FAMILY_ROOTS
            * std::mem::size_of::<worth_store_physical_format::DerivedFamilyRootEntry>())
            as u64
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        member: &AdmittedRootStepMemberView<'_>,
        source_root: &DurablePhysicalRootManifest,
        source_route: CurrentPhysicalRecordPlacement,
        source_bytes: &[u8],
        dropped: &[PersistedRecordIdentity],
        _format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, ReleasedDirectoryReplacementDenial> {
        Self::admit_inner(
            member.operation(),
            member.materialization(),
            member.record_bytes(0),
            member.record_bytes(1),
            source_root,
            source_route,
            source_bytes,
            dropped,
        )
    }

    /// The pending completion path retains the immutable plan, not C.9's
    /// earlier admitted-member roster. Exact projection pointer and canonical
    /// redo digest are required before any retained frame bytes are used.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_projection(
        projection: &PhysicalRedoProjection,
        plan: &ImmutablePhysicalRedoPlan,
        source_root: &DurablePhysicalRootManifest,
        source_route: CurrentPhysicalRecordPlacement,
        source_bytes: &[u8],
        dropped: &[PersistedRecordIdentity],
        _format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, ReleasedDirectoryReplacementDenial> {
        projection
            .semantics_admitted_redo_sha256()
            .filter(|digest| plan.admits_exact_member_redo_digest(projection, *digest))
            .ok_or(ReleasedDirectoryReplacementDenial::NotAdmittedReplacement)?;
        Self::admit_inner(
            projection.operation(),
            projection.materialization(),
            plan.admitted_projection_record_bytes(projection, 0),
            plan.admitted_projection_record_bytes(projection, 1),
            source_root,
            source_route,
            source_bytes,
            dropped,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn admit_inner(
        operation: [u8; 32],
        projection: &PersistedPhysicalRecoveryProjection,
        descriptor_bytes: Option<&[u8]>,
        directory_bytes: Option<&[u8]>,
        source_root: &DurablePhysicalRootManifest,
        source_route: CurrentPhysicalRecordPlacement,
        source_bytes: &[u8],
        dropped: &[PersistedRecordIdentity],
    ) -> Result<Self, ReleasedDirectoryReplacementDenial> {
        use ReleasedDirectoryReplacementDenial as Denial;
        let PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            head_effect: Some(_),
            directory_replacement: Some(replacement),
        } = projection.operation()
        else {
            return Err(Denial::NotAdmittedReplacement);
        };
        let [descriptor, directory] = projection.record_identities() else {
            return Err(Denial::NotAdmittedReplacement);
        };
        let [descriptor_route, next_route] = projection.placements() else {
            return Err(Denial::NotAdmittedReplacement);
        };
        let previous = replacement.expected_previous();
        let next = replacement.next();
        let source_watermark = previous
            .indexed_through_blob_publication()
            .ok_or(Denial::SourceBinding)?;
        if projection.source_root_generation() != source_root.generation()
            || source_root.derived_family_directory() != Some(previous)
            || source_route.record() != previous.directory_record()
            || dropped.binary_search(&source_watermark.record()).is_err()
            || dropped.binary_search(&previous.directory_record()).is_ok()
            || *descriptor != binding.record()
            || *directory != next.record().record()
            || descriptor_route.record() != *descriptor
            || next_route.record() != *directory
            || !matches!(descriptor_route, CurrentPhysicalRecordPlacement::Extent(_))
            || !matches!(next_route, CurrentPhysicalRecordPlacement::Extent(_))
            || binding.candidate_root_generation() != next.record().candidate_root_generation()
        {
            return Err(Denial::SourceBinding);
        }
        if <[u8; 32]>::from(Sha256::digest(source_bytes))
            != replacement.expected_previous_payload_sha256()
        {
            return Err(Denial::SourcePayload);
        }
        let old = DerivedFamilyRootDirectoryV1::decode(source_bytes)
            .map_err(|_| Denial::SourcePayload)?;
        if old.indexed_through_blob_publication() != Some(source_watermark) {
            return Err(Denial::SourcePayload);
        }
        let new_bytes = directory_bytes.ok_or(Denial::NewPayload)?;
        let new =
            DerivedFamilyRootDirectoryV1::decode(new_bytes).map_err(|_| Denial::NewPayload)?;
        if descriptor_bytes.is_none()
            || new.entries() != old.entries()
            || new.indexed_through_blob_publication().is_some()
            || new.indexed_through_quarantine() != old.indexed_through_quarantine()
            || next
                .indexed_through_quarantine()
                .is_some_and(|expected| new.indexed_through_quarantine() != expected)
            || <[u8; 32]>::from(Sha256::digest(new_bytes)) != next.record().record_payload_sha256()
        {
            return Err(Denial::NewPayload);
        }
        Ok(Self {
            operation,
            source_root: source_root.clone(),
            result_root: None,
            candidate_generation: binding.candidate_root_generation(),
            expected_result_latest: source_root
                .latest_blob_publication()
                .filter(|publication| dropped.binary_search(&publication.record()).is_err()),
            previous,
            previous_payload_sha256: replacement.expected_previous_payload_sha256(),
            source_route,
            next: DerivedFamilyRootDirectoryBinding::new(*directory, None),
            next_payload_sha256: next.record().record_payload_sha256(),
            next_route: *next_route,
            descriptor_record: *descriptor,
        })
    }

    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn candidate_generation(&self) -> u64 {
        self.candidate_generation
    }
    pub const fn descriptor_record(&self) -> PersistedRecordIdentity {
        self.descriptor_record
    }
    pub const fn source_binding(&self) -> DerivedFamilyRootDirectoryBinding {
        self.previous
    }
    pub const fn result_binding(&self) -> DerivedFamilyRootDirectoryBinding {
        self.next
    }
    /// Store's independent media rejoin of the C.9-admitted pair. The caller
    /// must obtain routes and frames from actual selected C.5 inventories.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_selected_media(
        &self,
        source_root: &DurablePhysicalRootManifest,
        source_route: CurrentPhysicalRecordPlacement,
        source_bytes: &[u8],
        result_root: &DurablePhysicalRootManifest,
        result_route: CurrentPhysicalRecordPlacement,
        result_bytes: &[u8],
    ) -> Result<(), ReleasedDirectoryReplacementDenial> {
        use ReleasedDirectoryReplacementDenial as Denial;
        if source_root != &self.source_root
            || self.result_root.as_ref() != Some(result_root)
            || source_root.derived_family_directory() != Some(self.previous)
            || result_root.derived_family_directory() != Some(self.next)
            || result_root.latest_blob_publication() != self.expected_result_latest
            || source_route != self.source_route
            || result_route != self.next_route
        {
            return Err(Denial::SourceBinding);
        }
        if <[u8; 32]>::from(Sha256::digest(source_bytes)) != self.previous_payload_sha256 {
            return Err(Denial::SourcePayload);
        }
        if <[u8; 32]>::from(Sha256::digest(result_bytes)) != self.next_payload_sha256 {
            return Err(Denial::NewPayload);
        }
        let old = DerivedFamilyRootDirectoryV1::decode(source_bytes)
            .map_err(|_| Denial::SourcePayload)?;
        let new =
            DerivedFamilyRootDirectoryV1::decode(result_bytes).map_err(|_| Denial::NewPayload)?;
        if old.indexed_through_blob_publication()
            != self.previous.indexed_through_blob_publication()
            || new.indexed_through_blob_publication().is_some()
            || new.entries() != old.entries()
            || new.indexed_through_quarantine() != old.indexed_through_quarantine()
        {
            return Err(Denial::NewPayload);
        }
        Ok(())
    }
    pub(crate) const fn source_root(&self) -> &DurablePhysicalRootManifest {
        &self.source_root
    }
    pub(crate) fn bind_result_root(&self, result_root: &DurablePhysicalRootManifest) -> Self {
        let mut bound = self.clone();
        bound.result_root = Some(result_root.clone());
        bound
    }
    pub(crate) const fn previous(&self) -> DerivedFamilyRootDirectoryBinding {
        self.previous
    }
    pub(crate) const fn source_route(&self) -> CurrentPhysicalRecordPlacement {
        self.source_route
    }
    pub(crate) const fn next(&self) -> DerivedFamilyRootDirectoryBinding {
        self.next
    }
    pub(crate) const fn next_route(&self) -> CurrentPhysicalRecordPlacement {
        self.next_route
    }
}
