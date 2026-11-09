//! The recorded correction support cursor names the known birth kind of an uncovered relation.
use super::*;
use crate::authored_ownership::native_checkpoint_tamper::{field, items, optional_field};
use std::collections::BTreeSet;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    history::{
        CommitId, RelationalAuthoredWriteDenial, RelationalDescriptiveTouch as Touch,
        RelationalDescriptiveTouchGraph as Graph,
    },
    identity::RelationId,
};
#[test]
fn recorded_support_refuses_uncovered_native_adjacency_with_its_exact_relation_and_kind() {
    type S = Model<48>;
    let host = super::super::correction_courts::install_definition::<
        S,
        super::super::coupled::multi::TwoProgram,
    >();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = super::super::coupled::multi::create(&host, &request, 10, vec![20]);
    let anchor = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    let edit = |value, key: &str| {
        let outcome = request
            .advanced()
            .mutate(Edit { id: 10, value })
            .without_source()
            .idempotency(&key.to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("{outcome:?}")
        };
        receipt
    };
    edit(3, "gap-initial");
    super::super::coupled::multi::settle(&host, &request, 10);
    let original = edit(7, "gap-source-seven");
    let source = host
        .runtime()
        .with_host_advancement(|phase| {
            let runtime = host.runtime();
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            runtime
                .branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
                .into_owned_head()
        })
        .unwrap();
    let source = host
        .runtime()
        .recorded_support_source_branch_for_test(source.branch(), &scope)
        .unwrap();
    super::super::coupled::multi::settle(&host, &request, 10);
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    let mut expected = None;
    let tampered = super::super::native_checkpoint_tamper::mutate(&checkpoint, |envelope| {
        let commit = field(envelope, "commit");
        let id = field(&envelope[commit.clone()], "commit_id");
        let commit: CommitId =
            rmp_serde::from_slice(&envelope[commit.start + id.start..commit.start + id.end])
                .unwrap();
        if commit <= original.commit_reference().commit_id {
            return None;
        }
        let origin = optional_field(envelope, "required_output_origin")?;
        let origin = &envelope[origin];
        if origin == [0xc0] {
            return None;
        }
        let owned = optional_field(origin, "source_scope_owned")?;
        if !rmp_serde::from_slice::<bool>(&origin[owned]).unwrap() {
            return None;
        }
        let allocations = field(envelope, "record_allocations");
        let allocations = &envelope[allocations];
        let mut allocated = BTreeSet::new();
        for item in items(allocations) {
            let allocation = &allocations[item];
            let record = field(allocation, "record");
            let record = &allocation[record];
            if let Some(relation) = optional_field(record, "Relation") {
                allocated.insert(rmp_serde::from_slice::<RelationId>(&record[relation]).unwrap());
            }
        }
        let span = field(envelope, "descriptive_touches");
        let mut graph: Graph = rmp_serde::from_slice(&envelope[span.clone()]).unwrap();
        let Graph::Exact(touches) = &mut graph else {
            return None;
        };
        let (relation, kind) = touches.iter().find_map(|touch| match touch {
            Touch::RelationMembership { relation, kind, .. } if allocated.contains(relation) => {
                Some((*relation, *kind))
            }
            _ => None,
        })?;
        // Remove lifecycle too: the diagnostic must recover the known kind from
        // the actual canonical patch's relation birth, not a surviving touch.
        touches.retain(|touch|!matches!(touch,Touch::RelationMembership{relation:id,..}|Touch::RelationLifecycle{relation:id,..} if *id==relation));
        expected = Some((commit, relation, kind));
        Some((span, rmp_serde::to_vec_named(&graph).unwrap()))
    });
    let (commit, relation, kind) =
        expected.expect("the real owned Preserve created private claim links");
    let recovered = super::super::coupled::history::reopen::<S>(tampered);
    let scope = request_scope();
    let principal = authenticate_operator(recovered.installed_schema(), &scope);
    let request = recovered.runtime().request(&principal, &scope);
    let result = recovered
        .runtime()
        .probe_recorded_adjacency_support_for_test(&source, anchor, kind, &scope)
        .unwrap();
    let Err(RelationalAuthoredWriteDenial::IncompleteAdjacencyHistory {
        commit: actual,
        relation: actual_relation,
        kind: actual_kind,
        ..
    }) = result
    else {
        panic!("recorded support must refuse the missing native coverage: {result:?}")
    };
    assert_eq!(
        (actual, actual_relation, actual_kind),
        (commit, Some(relation), Some(kind))
    );
    assert_eq!(
        request
            .query(query::Input { id: 10 })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        7
    );
}
