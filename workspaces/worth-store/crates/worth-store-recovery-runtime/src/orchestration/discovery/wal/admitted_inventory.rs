use worth_store::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, IntegrityAdmittedRecoveryWalFrame,
    IntegrityAdmittedRecoveryWalFrameView, IntegrityAdmittedRecoveryWalSegment,
};

pub(crate) struct AdmittedWalInventory {
    segments: crate::orchestration::NativeWalRoster<IntegrityAdmittedRecoveryWalSegment>,
}

impl Default for AdmittedWalInventory {
    fn default() -> Self {
        Self {
            segments: crate::orchestration::NativeWalRoster::empty(0),
        }
    }
}

impl AdmittedWalInventory {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        self.segments.as_slice().iter().try_fold(
            u64::try_from(self.segments.capacity())
                .ok()?
                .checked_mul(std::mem::size_of::<IntegrityAdmittedRecoveryWalSegment>() as u64)?,
            |bytes, segment| bytes.checked_add(segment.owned_heap_bytes()?),
        )
    }

    pub(super) fn prepare(
        owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
        count: usize,
    ) -> Result<Self, worth_store::physical_runtime::RecoveryWalAllocationDenial> {
        Ok(Self {
            segments: crate::orchestration::NativeWalRoster::with_capacity(owner, count)?,
        })
    }

    pub(crate) fn roster_charged_bytes(&self) -> u64 {
        self.segments.charged_bytes()
    }

    pub(super) fn push(&mut self, segment: IntegrityAdmittedRecoveryWalSegment) {
        self.segments.push_reserved(segment);
    }

    pub(crate) fn recoverable_frame_iter<'a>(
        &'a self,
        selected: &'a worth_store_recovery_physics::SelectedPhysicalWalTail,
    ) -> impl Iterator<Item = &'a IntegrityAdmittedRecoveryWalFrame> + Clone + 'a {
        self.recoverable_frame_view(selected).iter()
    }

    pub(crate) fn recoverable_frame_view<'a>(
        &'a self,
        selected: &'a worth_store_recovery_physics::SelectedPhysicalWalTail,
    ) -> IntegrityAdmittedRecoveryWalFrameView<'a> {
        IntegrityAdmittedRecoveryWalFrameView::from_selected_segments(
            self.segments.as_slice(),
            selected,
        )
    }

    pub(crate) fn cleanup_segments<'a>(
        &'a self,
        identities: impl IntoIterator<Item = WalSegmentArtifactIdentity> + 'a,
    ) -> impl Iterator<Item = IntegrityAdmittedRecoveryWalSegment> + 'a {
        identities
            .into_iter()
            .map(move |identity| self.segment(identity).clone())
    }

    fn segment(
        &self,
        identity: WalSegmentArtifactIdentity,
    ) -> &IntegrityAdmittedRecoveryWalSegment {
        self.segments
            .as_slice()
            .iter()
            .find(|segment| segment.inspection().identity() == identity)
            .expect("C.8 selection consumes a C.9-admitted WAL segment")
    }
}
