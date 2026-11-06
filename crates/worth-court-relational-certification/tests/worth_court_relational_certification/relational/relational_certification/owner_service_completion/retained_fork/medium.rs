//! An attached medium must never be contacted by the unsupported profile.
use super::*;
use std::sync::Arc;
use worth_relational::facade::{
    durability::*, runtime::RelationalRuntimeApi, schema::RelationalSchemaRegistry,
    storage::RelationalStorageUnavailableCause,
};

#[derive(Debug)]
struct UnusedMedium;
impl RelationalDurableColdSource for UnusedMedium {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, RelationalStorageUnavailableCause> {
        panic!("retained fork must not acquire medium members")
    }
    fn read_member(
        &self,
        _: RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        panic!("retained fork must not read medium members")
    }
    fn read_commit(
        &self,
        _: worth_relational::facade::publication::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        panic!("retained fork must not read the medium")
    }
}
impl RelationalDurableMedium for UnusedMedium {
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        Arc::new(UnusedMedium)
    }
    fn read_transition(
        &self,
        _: worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        panic!("no historical medium read")
    }
    fn publish(&self, _: &RelationalCanonicalCommitArtifact) -> DurablePublicationFate<String> {
        panic!("no unsupported commit publication")
    }
    fn publish_transition(
        &self,
        _: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        panic!("no unsupported transition publication")
    }
}

#[test]
fn attached_medium_refuses_before_reservation_transition_or_component_effect() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let source = empty_runtime();
    let source_services = source.owner_component_services();
    let committed = commit_through_services(execution, &source, &source_services, "source");
    let basis = source
        .admit_branch_basis(&source.main_branch_identity())
        .unwrap();
    let retained = source_services
        .fork_port()
        .admit_retained_fork_source(&basis)
        .unwrap();
    let durable = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .build_durable(Box::new(UnusedMedium))
        .unwrap();
    let fork = durable.fork_port();
    let durable_basis = durable
        .admit_branch_basis(&durable.main_branch_identity())
        .unwrap();
    assert!(matches!(
        fork.admit_retained_fork_source(&durable_basis),
        Err(RelationalForkDenial::RetainedSourceDurabilityUnsupported)
    ));
    assert!(matches!(
        fork.fork_retained_branch(execution, BranchId("unsupported".into()), retained),
        Err(RelationalBranchTransitionStop::Denied(
            RelationalForkDenial::RetainedSourceDurabilityUnsupported
        ))
    ));
    assert!(durable
        .branch_identity(&BranchId("unsupported".into()))
        .is_err());
    assert_eq!(
        durable
            .durability()
            .branch_transition_artifacts(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation
            )
            .len(),
        0
    );
    let reservation = fork
        .reserve_fork_target(BranchId("unsupported".into()))
        .unwrap();
    drop(reservation);
    source
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}
