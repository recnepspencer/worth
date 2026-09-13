use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

use crate::branch::{
    retention::SignalRetentionLedgerObservation, AdmittedSignalBranchBasis,
    ManagedSignalBranchReference, SignalBranchBasisDescriptor, SignalBranchObservation,
    SignalBranchTarget,
};
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchHandle;

use super::super::SignalBranchBasisPort;

pub(super) type TestRuntime = SignalRuntime<(), (), (), (), ()>;

pub(super) struct BasisPortWorld {
    pub(super) _runtime: TestRuntime,
    pub(super) branch_a: SignalBranchHandle,
    pub(super) branch_b: SignalBranchHandle,
    pub(super) basis_a: AdmittedSignalBranchBasis,
    pub(super) basis_b: AdmittedSignalBranchBasis,
    pub(super) port: SignalBranchBasisPort<(), (), ()>,
}

pub(super) fn basis_port_world() -> BasisPortWorld {
    let (mut runtime, branch_a, branch_b, basis_b) =
        super::super::super::tests::runtime_root::runtime_with_two_branches();
    let basis_a = runtime
        .observe_signal_branch_basis(branch_a.clone())
        .expect("the real runtime admits branch A before sealing");
    let (port, _, _) = runtime
        .owner_port_slots()
        .expect("the callback-free runtime seals into owner services");
    BasisPortWorld {
        _runtime: runtime,
        branch_a,
        branch_b,
        basis_a,
        basis_b,
        port,
    }
}

pub(super) fn issue_reference(
    port: &SignalBranchBasisPort<(), (), ()>,
    basis: &AdmittedSignalBranchBasis,
) -> ManagedSignalBranchReference {
    port.issue_managed_branch_reference(basis)
        .expect("the real port issues from a production-admitted basis")
}

pub(super) fn assert_retention_cleanup_with_identity_advance(
    before: &SignalRetentionLedgerObservation,
    after: &SignalRetentionLedgerObservation,
    issued_lease_count: u64,
) {
    assert_eq!(after.maximum_active_leases, before.maximum_active_leases);
    assert_eq!(after.used_capacity, before.used_capacity);
    assert_eq!(
        after.next_lease_id,
        before.next_lease_id + issued_lease_count
    );
    assert_eq!(after.admitted_lease_count, before.admitted_lease_count);
    assert_eq!(after.external_lease_count, before.external_lease_count);
    assert_eq!(
        after.reserved_admitted_lease_count,
        before.reserved_admitted_lease_count
    );
    assert_eq!(
        after.admitted_branch_total_count,
        before.admitted_branch_total_count
    );
    assert_eq!(
        after.reserved_branch_total_count,
        before.reserved_branch_total_count
    );
    assert_eq!(
        after.external_branch_total_count,
        before.external_branch_total_count
    );
    assert_eq!(
        after.external_target_total_count,
        before.external_target_total_count
    );
    assert_eq!(
        after.admitted_lease_identities,
        before.admitted_lease_identities
    );
    assert_eq!(
        after.external_lease_identities,
        before.external_lease_identities
    );
    assert_eq!(
        after.admitted_count_by_branch,
        before.admitted_count_by_branch
    );
    assert_eq!(
        after.reserved_count_by_branch,
        before.reserved_count_by_branch
    );
    assert_eq!(
        after.external_count_by_branch,
        before.external_count_by_branch
    );
    assert_eq!(
        after.external_count_by_target,
        before.external_count_by_target
    );
}

pub(super) fn advance_exact(
    port: &SignalBranchBasisPort<(), (), ()>,
    basis: &AdmittedSignalBranchBasis,
) -> SignalBranchObservation {
    let owner = port.upgrade_owner().expect("the sealed owner remains live");
    let admission = owner.admit().expect("adversarial movement admits once");
    let cell = owner
        .lookup_cell(&admission, basis.owner_branch_id())
        .expect("the exact basis names one installed cell");
    let outcome = cell
        .advance_exact::<(), (), _>(
            &admission,
            basis,
            &mut (),
            &super::super::super::SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the production cell performs one canonical movement");
    outcome.into_parts().0
}

pub(super) fn descriptor_with_target(
    template: &SignalBranchBasisDescriptor,
    target: SignalBranchTarget,
) -> SignalBranchBasisDescriptor {
    let mut value = serde_json::to_value(template).expect("the descriptor serializes");
    value["observation"]["target"] = serde_json::to_value(FoundationalBranchTarget::basis(target))
        .expect("the weak target serializes");
    serde_json::from_value(value).expect("the hostile weak descriptor remains well formed")
}

pub(super) fn target_changed_descriptor(
    template: &SignalBranchBasisDescriptor,
    graph_instance_id: impl Into<String>,
    definition_basis: u64,
    snapshot_id: Option<u64>,
) -> SignalBranchBasisDescriptor {
    let target = SignalBranchTarget::new(
        graph_instance_id,
        definition_basis,
        snapshot_id,
        template
            .observation()
            .target()
            .as_basis()
            .expect("the real descriptor has a basis target")
            .restore_snapshot_id(),
    )
    .expect("the hostile target uses a nonempty owner identity");
    descriptor_with_target(template, target)
}

pub(super) fn observation_with_generation(
    template: &SignalBranchBasisDescriptor,
    generation: u64,
) -> SignalBranchBasisDescriptor {
    let mut value = serde_json::to_value(template).expect("the descriptor serializes");
    value["observation"]["generation"] =
        serde_json::to_value(FoundationalBranchReferenceGeneration::new(generation))
            .expect("the hostile generation serializes");
    serde_json::from_value(value).expect("the hostile weak descriptor remains well formed")
}
