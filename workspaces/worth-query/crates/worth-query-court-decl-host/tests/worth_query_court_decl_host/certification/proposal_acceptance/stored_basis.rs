//! Transported comparison data must regain custody through the actual owners.
use super::*;
use worth_query_host::facade::{
    declaration::application_schema::ApplicationSchema,
    product::{
        WorthQueryExpectedProductBasisDescription as Description,
        WorthQueryExpectedProductBasisSelectionDenial as Denial,
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProductBranch,
    },
};

fn describe<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
) -> Description {
    application
        .with_host_advancement(|phase| {
            application
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .describe_basis()
        })
        .unwrap()
}

#[test]
fn serialized_expected_basis_readmits_owners_before_ordinary_candidate_adoption() {
    let host = publish_on_first_program();
    let source = host.current_world();
    // Only bytes survive the recording host call. No selection or lease does.
    let bytes = serde_json::to_vec(&describe(host.runtime(), source)).unwrap();
    let description: Description = serde_json::from_slice(&bytes).unwrap();
    let candidate = host
        .with_host_advancement(|phase| {
            let selected = host
                .runtime()
                .branches()
                .select_expected(&description, &phase)
                .unwrap();
            assert_eq!(selected.describe_basis(), description);
            host.runtime()
                .branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let branch = candidate.branch();
    assert_eq!(
        settle(set_retention(&host, branch, 3, 0x9181_1720)),
        RetentionVerdict::Performed(3)
    );
    let mut head = source;
    let previous = host
        .with_host_advancement(|phase| candidate.adopt(&mut head, &phase).unwrap())
        .unwrap();
    assert_eq!(head, branch);
    assert_eq!(previous, source);
    assert_eq!(read_retention(host.runtime(), head), 3);
    assert_eq!(read_retention(host.runtime(), previous), SEED_RETENTION);
    host.with_host_advancement(|phase| host.runtime().on_branch(head).close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn stored_expected_basis_refuses_movement_and_equal_payload_aba() {
    let host = publish_on_first_program();
    let source = host.current_world();
    let description = describe(host.runtime(), source);
    assert_eq!(
        settle(set_retention(&host, source, 3, 0x9181_1721)),
        RetentionVerdict::Performed(3)
    );
    for restored in [false, true] {
        if restored {
            assert_eq!(
                settle(set_retention(&host, source, SEED_RETENTION, 0x9181_1722)),
                RetentionVerdict::Performed(SEED_RETENTION)
            );
        }
        let failure = host
            .with_host_advancement(|phase| {
                host.runtime()
                    .branches()
                    .select_expected(&description, &phase)
                    .err()
                    .expect("the stored basis moved")
            })
            .unwrap();
        assert!(matches!(
            failure,
            Denial::Admission(WorthQueryProductBranchAdmissionDenial::Refused(
                WorthQueryProductBranchAdmissionRefusalKind::ObservationStaleSourceHead
            ))
        ));
    }
    let fresh = describe(host.runtime(), source);
    host.with_host_advancement(|phase| {
        host.runtime()
            .branches()
            .select_expected(&fresh, &phase)
            .unwrap()
    })
    .unwrap();
    assert_eq!(read_retention(host.runtime(), source), SEED_RETENTION);
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn expected_basis_rejects_foreign_and_unsupported_comparison_data() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let description = describe(host.runtime(), host.current_world());
    let denied = foreign
        .with_host_advancement(|phase| {
            foreign
                .runtime()
                .branches()
                .select_expected(&description, &phase)
                .err()
                .unwrap()
        })
        .unwrap();
    assert!(matches!(
        denied,
        Denial::Admission(WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner
        ))
    ));
    let mut encoded = serde_json::to_value(&description).unwrap();
    encoded["format_version"] = serde_json::json!(2);
    let unsupported: Description = serde_json::from_value(encoded).unwrap();
    let denied = host
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .select_expected(&unsupported, &phase)
                .err()
                .unwrap()
        })
        .unwrap();
    assert!(matches!(
        denied,
        Denial::UnsupportedDescriptionVersion {
            supported: 1,
            actual: 2
        }
    ));
    let mut encoded = serde_json::to_value(&description).unwrap();
    encoded["world"]["format_version"] = serde_json::json!(2);
    let unsupported: Description = serde_json::from_value(encoded).unwrap();
    let denied = host
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .select_expected(&unsupported, &phase)
                .err()
                .unwrap()
        })
        .unwrap();
    assert!(matches!(
        denied,
        Denial::Admission(WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::ExpectedDescriptionVersionUnsupported {
                supported: 1,
                actual: 2
            }
        ))
    ));
    assert_eq!(
        read_retention(host.runtime(), host.current_world()),
        SEED_RETENTION
    );
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn expected_basis_rejects_retained_native_history_inside_a_current_world_description() {
    let host = publish_on_first_program();
    let source = host.current_world();
    let old = serde_json::to_value(describe(host.runtime(), source)).unwrap();
    assert_eq!(
        settle(set_retention(&host, source, 3, 0x9181_1723)),
        RetentionVerdict::Performed(3)
    );
    let mut current = serde_json::to_value(describe(host.runtime(), source)).unwrap();
    current["native"] = old["native"].clone();
    let mixed: Description = serde_json::from_value(current).unwrap();
    let denied = host
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .select_expected(&mixed, &phase)
                .err()
                .unwrap()
        })
        .unwrap();
    // Readmission itself accepts retained exact history. Its separate currentness
    // receiver must reject it before any application selection or fork is issued.
    assert!(matches!(denied, Denial::NativeCurrentness(_)));
    assert_eq!(read_retention(host.runtime(), source), 3);
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}
