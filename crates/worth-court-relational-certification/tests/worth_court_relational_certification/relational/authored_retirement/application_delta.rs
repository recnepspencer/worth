//! Compare application output independently of Native custody consequences.
use super::*;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;

// The fixture declares only entity kind 1 and relation kind 2. All additional
// kinds belong to the public Native custody registrar, so compare exact
// application changes by kind while retaining both old and newly created rows.
pub(super) fn application_changes(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    before: &AdmittedRelationalBranchBasis,
    commit: &CommitResult,
) -> Vec<RecordRef> {
    let old = runtime
        .read_truth()
        .project_observation(&before.observation())
        .unwrap();
    let new = runtime
        .read_truth()
        .read_snapshot(&commit.snapshot, execution)
        .unwrap()
        .unwrap();
    commit
        .changed_records
        .iter()
        .filter(|record| match record {
            RecordRef::Entity(id) => {
                old.read_exact_entity_state(execution, *id, |m, _| m.kind_id)
                    .unwrap()
                    == Some(KindId(1))
                    || new
                        .entities()
                        .iter()
                        .any(|r| r.entity_id == *id && r.kind.kind_id == KindId(1))
            }
            RecordRef::Relation(id) => {
                old.read_exact_relation_state(execution, *id, |m, _| m.kind_id)
                    .unwrap()
                    == Some(KindId(2))
                    || new
                        .relations()
                        .iter()
                        .any(|r| r.relation_id == *id && r.kind.kind_id == KindId(2))
            }
        })
        .cloned()
        .collect()
}
