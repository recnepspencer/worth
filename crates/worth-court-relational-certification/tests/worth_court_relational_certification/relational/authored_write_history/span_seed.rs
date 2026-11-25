//! Public #382: a recovered runtime has no authored-write generation, and a
//! span seed covers exactly the original through the target. The oracle is the
//! live runtime's full generation over the same history.
use super::super::durable_publication::branch_history_publications::{
    fork, update_entity, update_entity_on_branch,
};
use super::{collect, cursor, keys, selected};
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::{
        BranchId, RelationalAuthoredWriteDenial as Denial, RelationalAuthoredWriteSpan as Span,
    },
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_publication::{create_entity_outcome, memory_runtime as runtime_with_test_schema},
    shared_relational_certification_test_execution_policy::serial_request,
};

/// Commits before the original; the seed must never walk them.
const EARLIER: usize = 200;
/// The older original sits after this many of the earlier commits.
const EARLY: usize = 100;
/// Spans after the original, in parent links.
const SPANS: [usize; 3] = [2, 12, 72];

fn basis(runtime: &RelationalRuntime, branch: &str) -> AdmittedRelationalBranchBasis {
    let identity = runtime.branch_identity(&BranchId(branch.into())).unwrap();
    runtime.admit_branch_basis(&identity).unwrap()
}

fn seed(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    original: &str,
    target: &str,
    links: usize,
) -> Result<Span, Denial> {
    let target = selected(runtime, &basis(runtime, target));
    runtime.seed_authored_write_span(execution, &basis(runtime, original), target, NonZeroUsize::new(links))
}

fn history(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    target: &str,
    entity: worth_relational::facade::identity::EntityId,
) -> Vec<worth_relational::facade::history::CommitId> {
    let (original, target) = (basis(runtime, "original"), basis(runtime, target));
    collect(runtime, execution, cursor(runtime, execution, &original, &target, entity)).0
}

#[test]
fn a_recovered_runtime_seeds_only_the_span_after_the_original() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "v-0");
    let entity = changed_entities(&created)[0];
    let mut outcomes = vec![];
    for n in 1..=EARLIER {
        outcomes.push(update_entity(&execution, &runtime, entity, &format!("early-{n}")));
        if n == EARLY {
            fork(&execution, &runtime, "early");
        }
    }
    fork(&execution, &runtime, "original");
    fork(&execution, &runtime, "diverged");
    let sibling = update_entity_on_branch(&execution, &runtime, entity, "sibling", BranchId("diverged".into()));
    for n in 1..=SPANS[2] {
        outcomes.push(update_entity(&execution, &runtime, entity, &format!("span-{n}")));
        if SPANS.contains(&n) {
            fork(&execution, &runtime, &format!("span-{n}"));
        }
    }
    let live = SPANS.map(|n| history(&runtime, execution, &format!("span-{n}"), entity));
    assert_eq!(live.each_ref().map(Vec::len), SPANS, "the live oracle sees each span's writes");
    runtime.durability_authority().checkpoint(&execution).unwrap();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered.durability_recovery().restore_native_checkpoint(&execution, &checkpoint).unwrap();
    for (n, expected) in SPANS.iter().zip(&live) {
        let target = format!("span-{n}");
        let cursor = recovered.authored_write_history_cursor(
            execution,
            &basis(&recovered, "original"),
            selected(&recovered, &basis(&recovered, &target)),
            &keys(entity),
        );
        assert!(
            matches!(cursor, Err(Denial::GenerationUnavailable)),
            "recovery leaves span {n} without a generation, and the cursor never seeds"
        );
        assert!(
            matches!(seed(&recovered, execution, "original", &target, n - 1), Err(Denial::SpanBoundReached)),
            "span {n} needs exactly {n} links"
        );
        assert_eq!(
            seed(&recovered, execution, "original", &target, *n).unwrap(),
            Span::Seeded { commits: n + 1 },
            "span {n} walks the original and its {n} descendants, never the {EARLIER} earlier commits"
        );
        assert_eq!(
            seed(&recovered, execution, "original", &target, 0).unwrap(),
            Span::Covered,
            "a later correction in the span walks nothing"
        );
        assert_eq!(&history(&recovered, execution, &target, entity), expected);
    }
    // Widen-only: an older original widens the span, and the narrower one stays covered.
    let widest = format!("span-{}", SPANS[2]);
    assert_eq!(
        seed(&recovered, execution, "early", &widest, 0).unwrap(),
        Span::Seeded { commits: EARLIER - EARLY + SPANS[2] + 1 }
    );
    assert_eq!(seed(&recovered, execution, "original", &widest, 0).unwrap(), Span::Covered);
    assert_eq!(&history(&recovered, execution, &widest, entity), &live[2]);
    // A diverged original is never reached: the walk ends at the root.
    assert!(matches!(
        seed(&recovered, execution, "diverged", "span-2", 0),
        Err(Denial::NotSelectedPrefix)
    ));
    release_test_commit_snapshot(&runtime, &created);
    release_test_commit_snapshot(&runtime, &sibling);
    for outcome in &outcomes {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
