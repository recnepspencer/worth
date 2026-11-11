//! Pure descriptive counterexamples; none constructs a runtime or live cell.
use super::*;
use crate::branch::{
    RelationalBranchRootDescriptor, RelationalBranchTarget, RelationalBranchVersion,
};
use worth_foundational::{
    FoundationalBranchReferenceGeneration as Generation, FoundationalBranchTarget,
};

fn cell(runtime: u64, branch: &str) -> RelationalBranchCellCheckpoint {
    RelationalBranchCellCheckpoint {
        runtime_instance_id: runtime,
        branch_id: BranchId(branch.into()),
        observation: relational_branch_observation(
            runtime,
            branch,
            FoundationalBranchTarget::basis(RelationalBranchTarget::new(
                runtime,
                4,
                3,
                vec![2],
                RelationalBranchRootDescriptor::new([1; 32], [2; 32]),
            )),
            Generation::new(2),
        )
        .unwrap(),
        truth_version: RelationalBranchVersion::new(3),
        fork_provenance: None,
        fork_source_branch_id: None,
    }
}
fn inspection(runtime: u64, wire: &'static [u8]) -> RelationalCheckpointInspection<'static> {
    RelationalCheckpointInspection {
        model: NativeCheckpointModelBytes {
            envelope: BTreeMap::from([("members".into(), b"same".as_slice())]),
            fields: BTreeMap::from([
                ("branch_cells".into(), wire),
                ("lifecycles".into(), b"live".as_slice()),
            ]),
            allocator: BTreeMap::new(),
        },
        branch_cells: vec![cell(runtime, "main")],
        symbol_identity_frontier: 0,
        frontiers: vec![],
        generations: vec![],
        current_publication_records: BTreeSet::new(),
        live_records: BTreeSet::new(),
    }
}
fn frames() -> (RelationalBranchIdentity, RelationalBranchIdentity) {
    (
        RelationalBranchIdentity::new(1_000_000_001, BranchId("main".into())),
        RelationalBranchIdentity::new(1_000_000_002, BranchId("main".into())),
    )
}
#[test]
fn rebound_comparison_preserves_exact_equality_and_every_noncell_byte() {
    let (source, recovered) = frames();
    let left = inspection(source.runtime_instance_id(), b"original-cell-encoding");
    let mut right = inspection(recovered.runtime_instance_id(), b"rebound-cell-encoding");
    assert_ne!(
        left.model, right.model,
        "ordinary atomicity comparison stays exact"
    );
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Ok(true)
    );
    for slot in ["envelope", "fields", "allocator"] {
        let map = match slot {
            "envelope" => &mut right.model.envelope,
            "fields" => &mut right.model.fields,
            "allocator" => &mut right.model.allocator,
            _ => unreachable!(),
        };
        map.insert("changed".into(), b"different");
        assert_eq!(
            left.model_matches_rebound_runtime(&right, &source, &recovered),
            Ok(false)
        );
        match slot {
            "envelope" => &mut right.model.envelope,
            "fields" => &mut right.model.fields,
            "allocator" => &mut right.model.allocator,
            _ => unreachable!(),
        }
        .remove("changed");
    }
}
#[test]
fn rebound_comparison_requires_the_complete_roster_and_actual_owner_frames() {
    let (source, recovered) = frames();
    let left = inspection(source.runtime_instance_id(), b"left");
    let mut right = inspection(recovered.runtime_instance_id(), b"right");
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &source),
        Ok(false)
    );
    right
        .branch_cells
        .push(cell(recovered.runtime_instance_id(), "sibling"));
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Ok(false)
    );
    right.branch_cells = vec![cell(recovered.runtime_instance_id(), "renamed")];
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Ok(false)
    );
    right.branch_cells = vec![cell(recovered.runtime_instance_id(), "main")];
    right.branch_cells[0].truth_version = RelationalBranchVersion::new(4);
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Ok(false)
    );
}
#[test]
fn rebound_comparison_keeps_full_fork_meaning_and_rejects_malformed_pairs() {
    let (source, recovered) = frames();
    let mut left = inspection(source.runtime_instance_id(), b"left");
    let mut right = inspection(recovered.runtime_instance_id(), b"right");
    left.branch_cells[0].fork_provenance =
        Some(cell(source.runtime_instance_id(), "origin").observation);
    left.branch_cells[0].fork_source_branch_id = Some(BranchId("origin".into()));
    right.branch_cells[0].fork_provenance =
        Some(cell(recovered.runtime_instance_id(), "origin").observation);
    right.branch_cells[0].fork_source_branch_id = Some(BranchId("origin".into()));
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Ok(true)
    );
    right.branch_cells[0].fork_provenance = Some(
        relational_branch_observation(
            recovered.runtime_instance_id(),
            "origin",
            FoundationalBranchTarget::empty(),
            Generation::new(2),
        )
        .unwrap(),
    );
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Ok(false)
    );
    right.branch_cells[0].fork_source_branch_id = None;
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Err(RelationalBranchCellDenial::CheckpointForkProvenanceMismatch)
    );
}
#[test]
fn rebound_comparison_validates_names_and_target_owners_before_normalization() {
    let (source, recovered) = frames();
    let left = inspection(source.runtime_instance_id(), b"left");
    let mut right = inspection(recovered.runtime_instance_id(), b"right");
    right.branch_cells[0].observation =
        cell(recovered.runtime_instance_id(), "wrong-name").observation;
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Err(RelationalBranchCellDenial::CheckpointObservationMismatch)
    );
    let correct = cell(recovered.runtime_instance_id(), "main").observation;
    right.branch_cells[0].observation = RelationalBranchReferenceObservation::new(
        correct.branch_id().clone(),
        cell(source.runtime_instance_id(), "main")
            .observation
            .target()
            .clone(),
        correct.generation(),
    );
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Err(RelationalBranchCellDenial::CheckpointObservationMismatch)
    );
    right.branch_cells = vec![cell(recovered.runtime_instance_id(), "main")];
    right.branch_cells[0].fork_provenance =
        Some(cell(recovered.runtime_instance_id(), "wrong-origin").observation);
    right.branch_cells[0].fork_source_branch_id = Some(BranchId("origin".into()));
    assert_eq!(
        left.model_matches_rebound_runtime(&right, &source, &recovered),
        Err(RelationalBranchCellDenial::CheckpointForkProvenanceMismatch)
    );
}
