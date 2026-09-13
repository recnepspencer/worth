use sha2::{Digest, Sha256};

use crate::branch::{SignalBranchObservation, SignalBranchRetirementReason};
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

const REFERENCE_PREFIX: &[u8] = b"WORTH-BRANCH-REFERENCE";
const TARGET_DOMAIN: &[u8] = b"worth.signal.branch-target";

pub(super) fn expected_terminal_basis_digest(
    branch: &SignalBranchHandle,
    observation: &SignalBranchObservation,
) -> String {
    expected_terminal_basis_digest_at_generation(
        branch,
        observation,
        observation.generation().get(),
    )
}

pub(super) fn expected_terminal_basis_digest_at_generation(
    branch: &SignalBranchHandle,
    observation: &SignalBranchObservation,
    generation: u64,
) -> String {
    let target = observation
        .target()
        .as_basis()
        .expect("retirement fixture carries a concrete target");
    let graph_instance_id = target.graph_instance_id().as_bytes();
    let identity = format!(
        "signal/{}:{}/{}:{}/{}:{}",
        graph_instance_id.len(),
        target.graph_instance_id(),
        branch.id.0.to_string().len(),
        branch.id.0,
        branch.name.len(),
        branch.name,
    );

    let mut target_bytes = Vec::new();
    append_length_prefixed(&mut target_bytes, graph_instance_id);
    target_bytes.extend_from_slice(&target.definition_basis().to_be_bytes());
    append_optional_u64(&mut target_bytes, target.snapshot_id());
    append_optional_u64(&mut target_bytes, target.restore_snapshot_id());

    let mut reference_bytes = Vec::new();
    reference_bytes.extend_from_slice(REFERENCE_PREFIX);
    reference_bytes.extend_from_slice(&[0, 1]);
    append_length_prefixed(&mut reference_bytes, identity.as_bytes());
    reference_bytes.push(1);
    append_length_prefixed(&mut reference_bytes, TARGET_DOMAIN);
    reference_bytes.extend_from_slice(&1_u16.to_be_bytes());
    append_length_prefixed(&mut reference_bytes, &target_bytes);
    reference_bytes.extend_from_slice(&generation.to_be_bytes());

    let json_bytes = serde_json::to_vec(&reference_bytes).expect("fixture bytes serialize");
    sha256_hex(&json_bytes)
}

pub(super) fn expected_closeout_digest(
    branch_id: SignalBranchId,
    parent_branch_id: SignalBranchId,
    forked_from_snapshot_id: Option<SignalSnapshotId>,
    terminal_head_snapshot_id: Option<SignalSnapshotId>,
    reason: SignalBranchRetirementReason,
    terminal_basis_digest: &str,
) -> String {
    let tuple_json = format!(
        "[{},{},{},{},\"{}\",\"{}\"]",
        branch_id.0,
        parent_branch_id.0,
        optional_snapshot_json(forked_from_snapshot_id),
        optional_snapshot_json(terminal_head_snapshot_id),
        reason_name(reason),
        terminal_basis_digest,
    );
    sha256_hex(tuple_json.as_bytes())
}

fn append_length_prefixed(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value);
}

fn append_optional_u64(output: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            output.push(1);
            output.extend_from_slice(&value.to_be_bytes());
        }
        None => output.push(0),
    }
}

fn optional_snapshot_json(value: Option<SignalSnapshotId>) -> String {
    value.map_or_else(|| "null".to_owned(), |snapshot| snapshot.0.to_string())
}

fn reason_name(reason: SignalBranchRetirementReason) -> &'static str {
    match reason {
        SignalBranchRetirementReason::Rejected => "Rejected",
        SignalBranchRetirementReason::Merged => "Merged",
        SignalBranchRetirementReason::Superseded => "Superseded",
        SignalBranchRetirementReason::DependencyCancellation => "DependencyCancellation",
        SignalBranchRetirementReason::ProjectionRebuild => "ProjectionRebuild",
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn independent_oracle_matches_fixed_terminal_and_closeout_vectors() {
    use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

    let branch = SignalBranchHandle {
        id: SignalBranchId(3),
        name: "child".to_owned(),
        parent_branch_id: Some(SignalBranchId(1)),
        head_snapshot_id: Some(SignalSnapshotId(5)),
    };
    let observation = crate::branch::signal_branch_observation(
        "17",
        branch.id.0,
        &branch.name,
        FoundationalBranchTarget::basis(
            crate::branch::SignalBranchTarget::new("17", 9, Some(4), None)
                .expect("fixed target is valid"),
        ),
        FoundationalBranchReferenceGeneration::new(2),
    )
    .expect("fixed observation is valid");
    let terminal = expected_terminal_basis_digest(&branch, &observation);
    assert_eq!(
        terminal,
        "92c5af1190197c9760ff46589b8dece7be80083c00e9b9750cd199084767c355"
    );
    assert_eq!(
        expected_closeout_digest(
            branch.id,
            SignalBranchId(1),
            Some(SignalSnapshotId(4)),
            branch.head_snapshot_id,
            SignalBranchRetirementReason::Rejected,
            &terminal,
        ),
        "ab162cfe55ae2d1000060a95995a943b9c6bd07118f80b9af2edc7e49f71c950"
    );
}
