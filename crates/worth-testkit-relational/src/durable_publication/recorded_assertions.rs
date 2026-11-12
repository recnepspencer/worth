//! Compare every durable axis without rewriting either original description.
use worth_foundational::FoundationalBranchTarget;
use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RecordedRelationalComponentReference},
    runtime::RelationalRuntime,
};

pub fn assert_recorded_basis(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    original: &RecordedRelationalComponentReference,
) {
    let actual = basis.descriptor();
    let recorded = original.descriptor();
    assert_eq!(actual.descriptor_version(), recorded.descriptor_version());
    assert_eq!(actual.branch_id(), recorded.branch_id());
    assert_eq!(actual.branch_id(), basis.identity().branch_id());
    assert_eq!(actual.truth_version(), recorded.truth_version());
    assert_eq!(actual.schema_commitment(), recorded.schema_commitment());
    assert_eq!(
        actual.visibility_commitment(),
        recorded.visibility_commitment()
    );
    assert_eq!(actual.posture(), recorded.posture());
    assert_eq!(
        actual.reference().generation(),
        recorded.reference().generation()
    );
    assert_eq!(actual.runtime_instance_id(), runtime.runtime_instance_id());
    assert_eq!(
        basis.identity().runtime_instance_id(),
        runtime.runtime_instance_id()
    );
    assert_ne!(actual.runtime_instance_id(), recorded.runtime_instance_id());
    assert_eq!(
        recorded.root_identity(),
        0,
        "canonical descriptions contain no process root allocation"
    );
    assert_eq!(
        actual.reference().branch_id().as_str(),
        format!(
            "relational/{}/{}",
            runtime.runtime_instance_id(),
            actual.branch_id().0
        )
    );
    assert_eq!(
        recorded.reference().branch_id().as_str(),
        format!(
            "relational/{}/{}",
            recorded.runtime_instance_id(),
            recorded.branch_id().0
        )
    );
    match (actual.reference().target(), recorded.reference().target()) {
        (FoundationalBranchTarget::Empty, FoundationalBranchTarget::Empty) => {
            assert_eq!(
                actual.root_identity(),
                0,
                "actual Empty root has no committed allocation"
            );
        }
        (
            FoundationalBranchTarget::Basis(actual_target),
            FoundationalBranchTarget::Basis(recorded_target),
        ) => {
            assert_eq!(
                actual_target.runtime_instance_id(),
                runtime.runtime_instance_id()
            );
            assert_eq!(
                recorded_target.runtime_instance_id(),
                recorded.runtime_instance_id()
            );
            assert_eq!(
                actual_target.selected_commit_id(),
                recorded_target.selected_commit_id()
            );
            assert_eq!(actual_target.version_id(), recorded_target.version_id());
            assert_eq!(
                actual_target.parent_commit_ids(),
                recorded_target.parent_commit_ids()
            );
            assert_eq!(actual_target.roots(), recorded_target.roots());
            assert_ne!(
                actual.root_identity(),
                recorded.root_identity(),
                "the actual admitted canonical basis retains its process allocation"
            );
        }
        _ => panic!("the original Empty/Canonical target discriminant must agree"),
    }
}
