//! An ordinary authored source edit outside both original discovery memberships.
use super::*;
pub(super) fn publish(
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<MembershipSchema, MembershipProgram>,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        MembershipSchema,
    >,
    key: u64,
    replacement: u64,
) {
    let query = request
        .query(PlanarRead {
            body_key: "isolated-a".into(),
        })
        .execute()
        .unwrap();
    assert_ne!(
        query.rows()[0].y,
        length(replacement),
        "unrelated edit must change Y for key={key}"
    );
    let source = query.observed_sources()[0].clone();
    drop(query);
    let outcome = request
        .advanced()
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "isolated-a".into(),
            operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                body_key: "isolated-a".into(),
                replacement_y: length(replacement),
            }]),
        }))
        .expect_source(source)
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "unrelated source must publish for key={key}: {outcome:?}"
    );
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "isolated-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(replacement)
    );
}

/// Independent authored fixture value before the first unrelated publication.
pub(super) fn assert_initial(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        MembershipSchema,
    >,
) {
    // PlanarRead's declared shape reads body and TWO successors. The existing
    // seed authors separate closed a->b->c->a rings. No derived marking answer
    // or result equality supplies this independent source-key oracle.
    let mut isolated = std::collections::BTreeSet::new();
    let mut retained = std::collections::BTreeSet::new();
    for (prefix, offset) in [
        ("anchor", 0),
        ("sibling", 20),
        ("remote", 40),
        ("isolated", 60),
    ] {
        for (suffix, seeded_y) in [("a", 1), ("b", 1), ("c", 10)] {
            let key = format!("{prefix}-{suffix}");
            let query = request
                .query(PlanarRead {
                    body_key: key.clone(),
                })
                .execute()
                .unwrap();
            assert_eq!(query.rows()[0].body_key, key);
            if prefix == "isolated" {
                assert_eq!(query.rows()[0].y, length(offset + seeded_y));
                assert!(isolated.insert(key));
            } else {
                assert!(retained.insert(key));
            }
        }
    }
    assert_eq!(isolated.len(), 3);
    assert_eq!(retained.len(), 9);
    assert!(
        isolated.is_disjoint(&retained),
        "whole seeded source rings are disjoint"
    );
    let discovery = request
        .query(PlanarDiscoveryRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let mut members = discovery.rows()[0].source_body_keys.clone();
    members.sort();
    assert_eq!(members, ["remote-b", "sibling-b", "sibling-c"]);
    assert!(members.iter().all(|key| retained.contains(key)));
    assert!(request
        .query(PlanarDiscoveryRead {
            body_key: "sibling-a".into()
        })
        .execute()
        .unwrap()
        .rows()[0]
        .source_body_keys
        .is_empty());
}
