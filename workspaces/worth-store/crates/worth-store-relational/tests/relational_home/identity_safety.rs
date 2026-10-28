//! Identity safety after recovery (D7): a recovered runtime never reissues a
//! record identity that was ever committed, including identities whose
//! records were later deleted. The check creates entities and a relation on
//! every recovered branch and requires each new identity to be outside the
//! script's complete committed history.

use std::collections::BTreeSet;

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::transactions::RecordRef;
use worth_store_relational::RelationalStoreHome;

use super::batches;
use super::workload::{self, Script};

/// Labels for the check's own creates, outside the script's step numbers.
const FIRST_FRESH_STEP: usize = 1_000_000;

pub(crate) fn assert_creates_are_fresh(
    home: &RelationalStoreHome,
    script: &Script,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) {
    let committed: BTreeSet<&RecordRef> = script.records.iter().collect();
    assert!(
        deletes_a_record(script),
        "{case}: the script's history must include a deleted record"
    );
    let runtime = home.runtime();
    let mut step = FIRST_FRESH_STEP;
    for name in ["main", "feature"] {
        let branch = BranchId(name.to_owned());
        if runtime.branch_identity(&branch).is_err() {
            continue;
        }
        let mut created = Vec::new();
        for _ in 0..2 {
            let outcome = workload::commit(runtime, execution, &branch, batches::create(step))
                .unwrap_or_else(|error| panic!("{case}: fresh create on {name}: {error:?}"));
            created.extend(outcome.changed_records.iter().cloned());
            step += 1;
        }
        let entities: Vec<_> = created
            .iter()
            .filter_map(|record| match record {
                RecordRef::Entity(entity) => Some(*entity),
                RecordRef::Relation(_) => None,
            })
            .collect();
        let [source, target] = entities[..] else {
            panic!("{case}: two creates on {name} issued {entities:?}");
        };
        let outcome = workload::commit(
            runtime,
            execution,
            &branch,
            batches::relate(source, target, step),
        )
        .unwrap_or_else(|error| panic!("{case}: fresh relation on {name}: {error:?}"));
        created.extend(outcome.changed_records.iter().cloned());
        step += 1;
        for record in &created {
            assert!(
                !committed.contains(record),
                "{case}: {name} reissued the committed identity {record:?}"
            );
        }
    }
}

/// Whether some branch held a record after one commit and no longer holds it
/// at the end of the script.
fn deletes_a_record(script: &Script) -> bool {
    let last = &script.states[script.states.len() - 1];
    script.states.iter().any(|state| {
        state.iter().any(|(branch, records)| {
            records.iter().enumerate().any(|(index, record)| {
                record != "None" && last.get(branch).is_some_and(|end| end[index] == "None")
            })
        })
    })
}
