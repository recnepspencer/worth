use worth_query_consumer_values::{PlanarVertex, PlanarVertexReplacement};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
        WorthQueryApplicationRequestQueryDenial,
    },
    application_invariants::EntityId,
    primary_graph::{
        LineageEventKind, MutationHandlerExecutionDenial, RecordStructuralChange,
        WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitReceipt,
        WorthQueryApplicationOutputProjectionDenial, WorthQueryApplicationOutputRole,
        WorthQueryCreateOutput, WorthQueryEntityResolutionDenialKind, WorthQueryPreserveOutput,
        WorthQueryRetireOutput, WorthQuerySourceExpectationDenialKind,
    },
};
use worth_query_topology_entry::{
    Body, PlanarMutationBinding, PlanarQuery, PlanarRead, VertexReplacement,
    VertexReplacementBinding,
};

use super::{adjust, length, mutate, read_y, require_planar_violation, source_version, Request};
use crate::ConsumerSchema;

mod source_aba;

use source_aba::{
    require_adjacency_aba_denial, require_aspect_aba_denial, require_child_aspect_aba_denial,
    require_nested_aspect_aba_denial,
};

pub(super) fn run(request: &Request<'_>) {
    let anchor = preserved_identity(request, "anchor-a", 2, 400);
    let retired = preserved_identity(request, "anchor-b", 1, 401);
    let input = replacement("anchor-b", "replacement-b", 11, 1);
    let source = observed_source(request, "anchor-a");
    require_missing_source(request, input.clone(), 405);
    let sibling = mutate(request, adjust("sibling-a", 22, 4096), 406);
    assert!(matches!(
        sibling,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let outcome = request
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&402_u64)
        .execute()
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        mut receipt,
        result,
    } = outcome
    else {
        panic!("a valid vertex replacement must publish its complete ring: {outcome:?}")
    };
    assert_eq!(result.replacement_key, "replacement-b");
    let correspondence = receipt.output_correspondence();
    let preserved = correspondence
        .entity(WorthQueryApplicationOutputRole::<
            VertexReplacementBinding<ConsumerSchema>,
            Body,
            WorthQueryPreserveOutput,
        >::new("anchor"))
        .unwrap()
        .entity_id();
    let created = correspondence
        .entity(WorthQueryApplicationOutputRole::<
            VertexReplacementBinding<ConsumerSchema>,
            Body,
            WorthQueryCreateOutput,
        >::new("replacement"))
        .unwrap()
        .entity_id();
    assert_eq!(
        correspondence
            .entity(WorthQueryApplicationOutputRole::<
                VertexReplacementBinding<ConsumerSchema>,
                (),
                WorthQueryCreateOutput,
            >::new("replacement"))
            .err(),
        Some(WorthQueryApplicationOutputProjectionDenial::EntityMismatch)
    );
    let deleted = correspondence
        .entity(WorthQueryApplicationOutputRole::<
            VertexReplacementBinding<ConsumerSchema>,
            Body,
            WorthQueryRetireOutput,
        >::new("retired"))
        .unwrap()
        .entity_id();
    assert_eq!(preserved, anchor);
    assert_eq!(deleted, retired);
    assert_ne!(created, retired);
    verify_committed_changes(&receipt, created, retired);
    assert!(receipt.take_performed_relational_product_change().is_some());
    assert!(receipt.take_performed_relational_product_change().is_none());
    assert_eq!(read_y(request, "replacement-b"), 1);
    assert_eq!(read_y(request, "anchor-a"), 2);
    assert_eq!(read_y(request, "anchor-c"), 10);
    require_absent(request, "anchor-b");

    let recovered = request
        .mutate(input.clone())
        .expect_source(source)
        .idempotency(&402_u64)
        .execute()
        .unwrap();
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(mut recovered) = recovered else {
        panic!("retry must recover the original replacement publication")
    };
    assert!(recovered.is_same_authoritative_commit(&receipt));
    assert_eq!(recovered.committed_changes(), receipt.committed_changes());
    assert!(recovered
        .take_performed_relational_product_change()
        .is_none());
    let drift = request
        .mutate(input)
        .expect_source(observed_source(request, "anchor-a"))
        .idempotency(&402_u64)
        .execute()
        .unwrap();
    assert!(matches!(
        drift,
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    ));
    assert_eq!(
        receipt.clone().committed_changes(),
        receipt.committed_changes()
    );
    // A later independent command resolves the replacement key through its own
    // selected decision read and preserves the actual owner-issued identity.
    assert_eq!(
        preserved_identity(request, "replacement-b", 1, 403),
        created
    );

    let before = source_version(request);
    let malformed = request
        .mutate(replacement("replacement-b", "rejected-replacement", 1, 12))
        .expect_source(observed_source(request, "anchor-a"))
        .idempotency(&404_u64)
        .execute()
        .unwrap();
    require_planar_violation(malformed);
    assert_eq!(source_version(request), before);
    assert_eq!(read_y(request, "replacement-b"), 1);
    assert_eq!(read_y(request, "anchor-a"), 2);
    assert_eq!(read_y(request, "anchor-c"), 10);
    assert_eq!(read_y(request, "sibling-a"), 22);
    require_absent(request, "rejected-replacement");

    require_aspect_aba_denial(request);
    require_child_aspect_aba_denial(request);
    require_nested_aspect_aba_denial(request);
    require_adjacency_aba_denial(request);
}

pub(super) fn require_foreign_source(
    request: &Request<'_>,
    source: worth_query_host::facade::primary_graph::WorthQueryObservedSource<PlanarQuery>,
) {
    let denial = request
        .mutate(replacement(
            "anchor-b",
            "foreign-source-must-not-publish",
            11,
            1,
        ))
        .expect_source(source)
        .idempotency(&412_u64)
        .execute()
        .expect_err("an observation from another application cannot authorize this mutation");
    let WorthQueryApplicationRequestMutationDenial::SourceExpectation(denial) = denial else {
        panic!("foreign source must retain its exact denial family: {denial:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQuerySourceExpectationDenialKind::ForeignApplication
    );
    require_absent(request, "foreign-source-must-not-publish");
}

fn require_missing_source(request: &Request<'_>, input: VertexReplacement, command: u64) {
    let denial = request
        .mutate(input)
        .idempotency(&command)
        .execute()
        .expect_err("a source-bound mutation cannot execute without its observation");
    let WorthQueryApplicationRequestMutationDenial::SourceExpectation(denial) = denial else {
        panic!("missing source must retain its exact denial family: {denial:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQuerySourceExpectationDenialKind::MissingExpectation
    );
}

fn require_source_changed(
    outcome: Result<
        WorthQueryApplicationMutationOutcome<
            worth_query_consumer_values::PlanarReplacementDenial,
            worth_query_consumer_values::PlanarVertexReplacementResult,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >,
) {
    let denial = outcome.expect_err("a stale source observation must deny publication");
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(denial),
    ) = denial
    else {
        panic!("stale source must remain an exact attempt denial: {denial:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
}

pub(super) fn observed_source(
    request: &Request<'_>,
    key: &str,
) -> worth_query_host::facade::primary_graph::WorthQueryObservedSource<PlanarQuery> {
    let result = request
        .query(PlanarRead {
            body_key: key.to_owned(),
        })
        .execute()
        .expect("the mutation source query must complete");
    result
        .observed_sources()
        .first()
        .expect("the selected row carries its source stamp")
        .clone()
}

fn preserved_identity(request: &Request<'_>, key: &str, y: u64, command: u64) -> EntityId {
    let mut input = adjust(key, y, 4096);
    input.scope_key = key.to_owned();
    let outcome = mutate(request, input, command);
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("the identity observation must come from a real selected command: {outcome:?}")
    };
    receipt
        .output_correspondence()
        .entity(WorthQueryApplicationOutputRole::<
            PlanarMutationBinding<ConsumerSchema>,
            Body,
            WorthQueryPreserveOutput,
        >::new("anchor"))
        .unwrap()
        .entity_id()
}

fn verify_committed_changes(
    receipt: &WorthQueryApplicationCommitReceipt,
    created: EntityId,
    retired: EntityId,
) {
    let changes = receipt.committed_changes();
    assert_eq!(changes.commit_reference(), receipt.commit_reference());
    let structural = changes.entity_changes().collect::<Vec<_>>();
    assert!(structural.contains(&(created, RecordStructuralChange::Created)));
    assert!(structural.contains(&(retired, RecordStructuralChange::Deleted)));
    let events = changes.lineage_events();
    assert!(
        !events.is_empty(),
        "replacement must co-commit actual lineage"
    );
    for event in events {
        assert_eq!(event.commit(), receipt.commit_reference());
    }
    // Include framework entities in both counts. No event order or numeric ID
    // is used to manufacture an entity-to-lineage association.
    let creates = events
        .iter()
        .filter(|event| event.kind() == LineageEventKind::Create)
        .count();
    let retires = events
        .iter()
        .filter(|event| event.kind() == LineageEventKind::Retire)
        .count();
    assert_eq!(
        creates,
        structural
            .iter()
            .filter(|(_, change)| *change == RecordStructuralChange::Created)
            .count()
    );
    assert_eq!(
        retires,
        structural
            .iter()
            .filter(|(_, change)| *change == RecordStructuralChange::Deleted)
            .count()
    );
    assert!(creates >= 1);
    assert_eq!(retires, 1);
    for event in events
        .iter()
        .filter(|event| event.kind() == LineageEventKind::Retire)
    {
        assert_eq!(event.sources().len(), 1);
        assert!(event.targets().is_empty());
    }
}

fn replacement(retired: &str, key: &str, x: u64, y: u64) -> VertexReplacement {
    VertexReplacement {
        scope_key: "anchor-a".to_owned(),
        replacement: PlanarVertexReplacement {
            retired_key: retired.to_owned(),
            next_key: "anchor-c".to_owned(),
            replacement: PlanarVertex {
                body_key: key.to_owned(),
                x: length(x),
                y: length(y),
            },
        },
    }
}

fn require_absent(request: &Request<'_>, key: &str) {
    match request
        .query(PlanarRead {
            body_key: key.to_owned(),
        })
        .execute()
    {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial)) => {
            assert_eq!(
                denial.kind(),
                WorthQueryEntityResolutionDenialKind::UnknownEntity
            );
        }
        Err(denial) => panic!("another denial cannot prove vertex absence: {denial:?}"),
        Ok(_) => panic!("the absent vertex must not be selectable: {key}"),
    }
}
