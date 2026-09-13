#[path = "retention_preflight/denials.rs"]
mod denials;
#[path = "retention_preflight/historical.rs"]
mod historical;

use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

use crate::branch::{
    admit_runtime_signal_branch_observation, signal_branch_observation, AdmittedSignalBranchBasis,
    SignalBranchTarget,
};
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::super::{SignalOwner, SignalOwnerOperationAdmission};

type TestOwner = SignalOwner<(), (), ()>;

fn owner_admitted_exact_target(
    owner: &TestOwner,
    admission: &SignalOwnerOperationAdmission<'_>,
    handle: &SignalBranchHandle,
    retention_branch_id: SignalBranchId,
    definition_basis: u64,
    snapshot_id: Option<SignalSnapshotId>,
) -> AdmittedSignalBranchBasis {
    let target = SignalBranchTarget::new(
        owner.runtime_instance_id().to_string(),
        definition_basis,
        snapshot_id.map(|snapshot| snapshot.0),
        None,
    )
    .expect("hostile exact-target fixture is structurally valid");
    let observation = signal_branch_observation(
        owner.runtime_instance_id().to_string(),
        handle.id.0,
        &handle.name,
        FoundationalBranchTarget::Basis(target),
        FoundationalBranchReferenceGeneration::new(0),
    )
    .expect("hostile observation is structurally valid");
    let retention = owner
        .acquire_admitted_retention(admission, retention_branch_id)
        .expect("the fixture uses a real owner-issued admitted lease");
    admit_runtime_signal_branch_observation(observation, handle.id, retention)
}
