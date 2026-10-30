pub(crate) use {
    worth_store_physical_backend::BackendCapabilityAdmissionRequest,
    worth_store_physical_backend::BackendCapabilityEvidenceBasis,
    worth_store_physical_backend::BackendCapabilitySupportSet,
    worth_store_physical_backend::BackendMediaAssumptionSet,
    worth_store_physical_backend::BackendRebindTriggers,
    worth_store_physical_backend::BackendTargetProfile,
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority,
};
pub(crate) use {
    worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalPageId,
    worth_store_physical_format::PhysicalReclaimRegion,
    worth_store_physical_format::PhysicalRecordSlot,
    worth_store_physical_format::PhysicalReference,
    worth_store_physical_format::PhysicalReferenceAuthority,
    worth_store_physical_format::PhysicalSegmentId,
    worth_store_physical_format::ReclaimedByteInterpretation,
};
pub(crate) use {
    worth_store_physical_isolation::admit_seed_stable_read_plan,
    worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot,
    worth_store_physical_isolation::CompactProtectedReferenceSet,
    worth_store_physical_isolation::CurrentPhysicalRoot,
    worth_store_physical_isolation::ExecutedReachabilityEvidence,
    worth_store_physical_isolation::HazardLeaseEpochIndexSnapshot,
    worth_store_physical_isolation::HazardLeaseTable,
    worth_store_physical_isolation::HazardLeaseTableCapacity,
    worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics,
    worth_store_physical_isolation::PostProtectionPhysicalReadObservation,
    worth_store_physical_isolation::ProtectedPhysicalReferenceSet,
    worth_store_physical_isolation::ProtectedReferenceLease,
    worth_store_physical_isolation::PublishedReaderHazard,
    worth_store_physical_isolation::ReadPlanAdmissionScratchArena,
    worth_store_physical_isolation::ReclaimCandidateSet,
    worth_store_physical_isolation::ReclaimEligibilityProof,
    worth_store_physical_isolation::ReleasedOldReachability,
    worth_store_physical_isolation::S6ReclaimReachabilityRemovalEvidence,
    worth_store_physical_isolation::TraversalAdmissionGuard,
    worth_store_physical_isolation::UnprotectedReadIntent,
};
pub(crate) use {
    worth_store_reclaim_policy::AdmittedReclaimPolicy,
    worth_store_reclaim_policy::PhysicalStoreReclaimPolicyExecutor,
    worth_store_reclaim_policy::ReclaimPermit, worth_store_reclaim_policy::ReclaimPolicyAdmission,
    worth_store_reclaim_policy::ReclaimPolicyExecutionObservation,
    worth_store_reclaim_policy::ReclaimPolicyExecutionReceipt,
    worth_store_reclaim_policy::ReclaimPolicyExecutionRequest,
    worth_store_reclaim_policy::ReclaimPolicyExecutionSession,
    worth_store_reclaim_policy::ReclaimPolicyProofAuthority,
    worth_store_reclaim_policy::ReclaimPolicyReachabilityDenial,
    worth_store_reclaim_policy::ReclaimPolicyReachabilityProof,
    worth_store_reclaim_policy::ReclaimPolicyRequest,
    worth_store_reclaim_policy::ReclaimPolicySecurityScope,
    worth_store_reclaim_policy::ReclaimPolicyViolation,
    worth_store_reclaim_policy::StoreOwnedReclaimPolicyExecution,
};

pub(crate) use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout,
};

#[derive(Clone)]
pub(crate) struct ObservingBackend {
    pub(crate) observation: ReclaimPolicyExecutionObservation,
}

pub(crate) struct S6ReclaimFixture {
    pub(crate) candidates: ReclaimCandidateSet,
    pub(crate) released: ReleasedOldReachability,
    pub(crate) root: CurrentPhysicalRoot,
    pub(crate) live_lease: ProtectedReferenceLease,
}

impl PhysicalStoreReclaimPolicyExecutor for ObservingBackend {
    type Error = ();

    fn execute_reclaim_policy(
        &mut self,
        request: ReclaimPolicyExecutionRequest,
    ) -> Result<ReclaimPolicyExecutionObservation, Self::Error> {
        let _ = request;
        Ok(self.observation.clone())
    }
}

impl S6ReclaimFixture {
    pub(crate) fn new(generation: u64) -> Self {
        let authority = physical_authority_from_complete_closeout();
        let root = current_root_from_authority(&authority);
        let reference = current_generation_page_reference(generation);
        let references = worth_store_physical_isolation::ProtectedPhysicalReferenceSet::from_current_generation_refs_with_scratch(
            [reference],
            worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(1),
        )
        .unwrap();
        let observed_references = references.clone();
        let compact = worth_store_physical_isolation::CompactProtectedReferenceSet::from_reference_set_with_scratch(
            references.clone(),
            worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(1),
        )
        .unwrap();
        let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
            root, references, 4096,
        )
        .with_release_semantics(
            worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
        );
        let hazard =
            worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, intent)
                .unwrap();
        let live_lease =
            worth_store_physical_isolation::ProtectedReferenceLease::from_reader_hazard(
                &hazard,
                compact.clone(),
            )
            .unwrap();
        let observed =
            worth_store_physical_isolation::PostProtectionPhysicalReadObservation::from_authority_after_hazard_publication(
                &authority,
                &hazard,
                root,
                observed_references,
            )
            .unwrap();
        let validated = hazard
            .observe_authority_after_publication(&authority, observed)
            .unwrap()
            .validate()
            .unwrap();
        let receipt = worth_store_physical_isolation::TraversalAdmissionGuard::from_validated_root(validated)
            .admit(worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(1))
            .unwrap();
        let read_release = admit_seed_stable_read_plan(receipt.into_cursor().finish())
            .unwrap()
            .into_execution_ready_handle()
            .release();
        let released =
            worth_store_physical_isolation::OldReachabilityPreservation::from_protected_footprint(
                compact.declared_footprint_basis(),
            )
            .unwrap()
            .admit_release(read_release)
            .unwrap();
        let candidates =
            worth_store_physical_isolation::ReclaimCandidateSet::from_released_old_reachability(
                released, &compact,
            )
            .unwrap();
        Self {
            candidates,
            released,
            root,
            live_lease,
        }
    }

    pub(crate) fn executed_reachability(&self) -> ExecutedReachabilityEvidence {
        worth_store_physical_isolation::ExecutedReachabilityEvidence::from_released_old_reachability(
            self.released,
            self.candidates.clone(),
        )
        .unwrap()
    }

    pub(crate) fn live_hazard_snapshot(&self) -> HazardLeaseEpochIndexSnapshot {
        let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
            worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
        );
        table.acquire(self.root, self.live_lease.clone()).unwrap();
        table.live_index_snapshot()
    }
}

pub(crate) fn admitted_policy_for_region(
    region: PhysicalReclaimRegion,
    interpretation: ReclaimedByteInterpretation,
) -> AdmittedReclaimPolicy {
    let backend = admitted_backend();
    let authority =
        worth_store_reclaim_policy::ReclaimPolicyProofAuthority::for_admitted_backend(&backend);
    worth_store_reclaim_policy::ReclaimPolicyAdmission::admit(
        authority,
        base_real_chain_request(
            region,
            real_reachability_for_region(region.reference().generation().get(), region),
        )
        .with_posture(authority.cold_tier_io_posture(interpretation).unwrap())
        .with_security_scope(internal_security_scope())
        .with_reclaim_permit(worth_store_reclaim_policy::ReclaimPermit::new(1).unwrap())
        .with_later_handoff_policy(authority.non_claim_later_handoff()),
    )
    .unwrap()
}

pub(crate) fn base_real_chain_request(
    region: PhysicalReclaimRegion,
    reachability: ReclaimPolicyReachabilityProof,
) -> ReclaimPolicyRequest {
    worth_store_reclaim_policy::ReclaimPolicyRequest::new()
        .for_region(region)
        .with_reachability(reachability)
}

pub(crate) fn execute_policy_with_observation(
    policy: AdmittedReclaimPolicy,
    observation: ReclaimPolicyExecutionObservation,
) -> Result<ReclaimPolicyExecutionReceipt, ReclaimPolicyViolation> {
    let mut executor = ObservingBackend { observation };
    worth_store_reclaim_policy::ReclaimPolicyExecutionSession::for_store_backend(
        &mut executor,
        worth_store_reclaim_policy::StoreOwnedReclaimPolicyExecution::for_certification_test_authority(),
    )
    .execute(policy)
    .unwrap()
}

pub(crate) fn internal_security_scope() -> ReclaimPolicySecurityScope {
    worth_store_reclaim_policy::ReclaimPolicySecurityScope::from_admitted_scope(
        &worth_store_security::admitted_store_internal_security_scope_for_io_qos_test(),
    )
}

pub(crate) fn real_reachability_for_region(
    generation: u64,
    region: PhysicalReclaimRegion,
) -> ReclaimPolicyReachabilityProof {
    let world =
        crate::io_scheduling::reclaim_policy::reclaim_support::S6ReclaimFixture::new(generation);
    let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        worth_store_physical_isolation::HazardLeaseTable::with_capacity(
            worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
        )
        .live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();
    let removal = proof.admit_reachability_removal().unwrap();
    reachability_from_physical_isolation_removal(
        removal.lower_for_io_qos_reclaim_policy(region).unwrap(),
        region,
    )
    .unwrap()
}

pub(crate) fn reachability_from_physical_isolation_removal(
    evidence: S6ReclaimReachabilityRemovalEvidence,
    requested_region: PhysicalReclaimRegion,
) -> Result<ReclaimPolicyReachabilityProof, ReclaimPolicyReachabilityDenial> {
    worth_store_reclaim_policy::ReclaimPolicyReachabilityProof::from_physical_isolation_reclaim_reachability_removal(
        evidence,
        requested_region,
    )
}

pub(crate) fn admitted_backend() -> worth_store_physical_backend::AdmittedBackendCapabilityWitness {
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority::store_owned()
        .admit_backend_capability(worth_store_physical_backend::BackendCapabilityAdmissionRequest::new(
            worth_store_physical_backend::BackendTargetProfile::PosixFileFsyncDirSync,
            worth_store_physical_backend::BackendCapabilityEvidenceBasis::certified_backend_profile(),
            worth_store_physical_backend::BackendCapabilitySupportSet::all_supported(),
            worth_store_physical_backend::BackendMediaAssumptionSet::platform_file_defaults()
                .with_trim_posture()
                .with_punch_hole_posture()
                .with_sparse_posture()
                .with_cold_tier_io_posture(),
            worth_store_physical_backend::BackendRebindTriggers::kernel_filesystem_mount_firmware_and_backend(),
        ))
        .unwrap()
}

pub(crate) fn backend_without_reclaim_posture(
) -> worth_store_physical_backend::AdmittedBackendCapabilityWitness {
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority::store_owned()
        .admit_backend_capability(worth_store_physical_backend::BackendCapabilityAdmissionRequest::new(
            worth_store_physical_backend::BackendTargetProfile::PosixFileFsyncDirSync,
            worth_store_physical_backend::BackendCapabilityEvidenceBasis::certified_backend_profile(),
            worth_store_physical_backend::BackendCapabilitySupportSet::all_supported(),
            worth_store_physical_backend::BackendMediaAssumptionSet::platform_file_defaults(),
            worth_store_physical_backend::BackendRebindTriggers::kernel_filesystem_mount_firmware_and_backend(),
        ))
        .unwrap()
}

pub(crate) fn region_for_generation(generation: u64) -> PhysicalReclaimRegion {
    worth_store_physical_format::PhysicalReclaimRegion::new(
        reference_for_generation(generation),
        4096,
    )
    .unwrap()
}

pub(crate) fn reference_for_generation(generation: u64) -> PhysicalReference {
    let cell =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .slot_cell(
                worth_store_physical_format::PhysicalSegmentId::from_raw(17).unwrap(),
                worth_store_physical_format::PhysicalPageId::from_raw(23).unwrap(),
                worth_store_physical_format::PhysicalRecordSlot::from_raw(1).unwrap(),
            )
            .with_slot_generation(
                worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap(),
            );
    worth_store_physical_format::PhysicalReferenceAuthority::for_canonical_physical_format()
        .admit_page_slot(cell)
        .reference()
}
