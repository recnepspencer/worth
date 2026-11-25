//! A successor round retracts its chain over a candidate whose aggregate
//! owner projects derived graph laws: another chain's `uses` link to a
//! record the round re-creates may not fail that projection mid-round, and
//! one to a record the round drops still refuses `RetiredReference`.
use super::multi_chain::{outputs_of, round, two_routes};
use super::*;

/// Chain 0 creates `b`; chain 1 creates `y` with a `uses` link to `b`; then,
/// with `second`, chain 0's next round creates `second`.
fn relinked(second: Option<&'static [&'static str]>) -> Result<Run, TransactionCommitError> {
    run_with(
        |execution| two_routes(World::with_derived_law(execution)),
        false,
        &|world, execution, candidate| {
            round(world, execution, candidate, 0, &["b"], Vec::new())?;
            let b = outputs_of(candidate, execution, world, "owned")?[0];
            let uses = WorkerIntentBatch::new("uses").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: USES,
                    client_key: ClientKey::raw("y-uses-b"),
                    source: EntityReference::Created(target("y")),
                    target: EntityReference::Existing(b),
                    fields: AspectFieldPatch::default(),
                }),
            ));
            round(world, execution, candidate, 1, &["y"], vec![uses])?;
            match second {
                Some(keys) => round(world, execution, candidate, 0, keys, Vec::new()),
                None => Ok(()),
            }
        },
    )
}

#[test]
fn a_round_re_creating_a_record_another_chain_links_commits() {
    let chains = relinked(Some(&["b"])).expect("b keeps its identity, so y's link holds");
    let head = relinked(None).unwrap();
    assert_eq!(
        chains.outputs["b"], head.outputs["b"],
        "b keeps its identity"
    );
    assert_eq!(chains.observed, head.observed);
}

#[test]
fn a_round_dropping_a_record_another_chain_links_is_a_retired_reference() {
    let cause = refusal(relinked(Some(&["c"])).err().expect("y still links b"));
    assert_eq!(cause, postponed(Reason::RetiredReference));
}
