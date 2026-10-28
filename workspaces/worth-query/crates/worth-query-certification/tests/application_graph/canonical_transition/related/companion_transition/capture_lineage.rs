//! A capture policy that does not descend from the funding lease refuses at
//! entry: no authoring, no commit, no charge, and the checkpoint unchanged.
//! A foreign lease is nested-lease misuse; naming no lease is unleased backing.
use super::*;
use worth_execution::ExecutionResourceLease;
use worth_query_host::facade::{
    application_contribution::WorthQueryManagedComputationResourceDenial as Resource,
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenialKind as Kind,
        WorthQueryProviderSessionDenialKind as SessionKind,
    },
};

fn assert_refused_at_entry(
    denied: Result<Target, installation::WorthQueryInMemoryApplicationDenial>,
    unleased: bool,
) {
    let installation::WorthQueryInMemoryApplicationDenial::Graph(denial) = denied
        .err()
        .expect("a capture outside the funding lineage must refuse at entry")
    else {
        panic!("the refusal is the typed entry refusal, not a later stage")
    };
    if unleased {
        assert!(
            matches!(
                denial.kind(),
                Kind::ExecutionDenied {
                    kind: SessionKind::ResourceEnvelopeMismatch
                }
            ),
            "{denial:?}"
        );
    } else {
        assert!(
            matches!(
                denial.kind(),
                Kind::ExecutionDenied {
                    kind: SessionKind::ExecutionResource {
                        denial: Resource::NestedLeaseMisuse,
                        ..
                    }
                }
            ),
            "{denial:?}"
        );
    }
}

#[test]
fn sibling_foreign_and_unnamed_capture_policies_refuse_before_work_or_charge() {
    let (original, settlement) = original(model::authored_value());
    let ids = companion_ids(&settlement);
    let source_id = old::reads::source_id(&original);
    let checkpoint = old::host::capture(&original);
    let bytes = checkpoint.bytes().to_vec();
    let funding_request = || {
        super::super::super::capture_child::funding(worth_execution::CancellationToken::new(), None)
    };
    let root = ExecutionResourceLease::serial(funding_request()).unwrap();
    let funding = root.child(funding_request()).unwrap();
    // Beside the funding lease under the same root.
    let sibling = root.child(funding_request()).unwrap();
    // Another serial authority's lineage, and another process authority's.
    let foreign_serial = ExecutionResourceLease::serial(funding_request()).unwrap();
    let foreign_authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test()
            .request_lease(funding_request())
            .unwrap();
    let leases = [
        &root,
        &funding,
        &sibling,
        &foreign_serial,
        &foreign_authority,
    ];
    successor::output::reset_decodes();
    let mut callbacks = 0;
    for capture in [
        installation::WorthQueryCheckpointCapturePolicy::Execution(&sibling),
        installation::WorthQueryCheckpointCapturePolicy::Execution(&root),
        installation::WorthQueryCheckpointCapturePolicy::Execution(&foreign_serial),
        installation::WorthQueryCheckpointCapturePolicy::Execution(&foreign_authority),
        // Naming no lease never silently inherits the funding lease.
        installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
    ] {
        let unleased = matches!(
            capture,
            installation::WorthQueryCheckpointCapturePolicy::SystemAllocation
        );
        assert_refused_at_entry(
            transition_funded(checkpoint.clone(), &funding, capture, &mut callbacks),
            unleased,
        );
        assert_eq!(callbacks, 0, "no authoring ran");
        assert_eq!(successor::output::decodes(), 0);
        for lease in leases {
            assert_eq!(lease.charged_memory_bytes(), 0, "nothing was charged");
        }
        assert_eq!(checkpoint.bytes(), bytes);
    }
    assert_eq!(old::reads::read(&original), model::authored_value());

    // The funding lease itself, or any descendant of it, caps the capture.
    let target = transition_funded(
        checkpoint,
        &funding,
        installation::WorthQueryCheckpointCapturePolicy::Execution(&funding),
        &mut callbacks,
    )
    .expect("the funding lease caps its own capture");
    assert_eq!(callbacks, 1);
    assert_eq!(new::reads::source_id(&target), source_id);
    assert_target(&target, ids);
}
