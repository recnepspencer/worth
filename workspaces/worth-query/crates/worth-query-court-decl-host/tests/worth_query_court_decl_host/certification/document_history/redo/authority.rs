//! Metadata refusal precedes any child lookup or cursor/buffer diagnostic.
use super::*;
use std::num::NonZeroU64;
use worth_query_host::facade::application_entry::WorthQueryHistoryContextRefreshLimits;
use worth_query_testkit_document_retention::facade::document_retention_model::record_history::capability;

fn input() -> capability::Input {
    capability::Input {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

#[test]
fn redo_hints_require_current_metadata_before_protected_diagnostics_and_after_revocation() {
    let host = publish_on_first_program();
    let (public, _) = forked(&host);
    let mut document = super::super::metadata::governed(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let foreign = public
        .redo_choices(&request, limits(1), None)
        .unwrap()
        .children()
        .continuation()
        .unwrap();
    let observed = document.observe_history_enrollment_for_test();
    let tiny = Limits {
        maximum_owned_bytes: NonZeroUsize::MIN,
        ..limits(1)
    };
    let denied = document
        .redo_choices(&request, tiny, Some(&foreign))
        .unwrap_err();
    let Failure::Document(Native::Metadata(
        worth_query_host::facade::primary_graph::WorthQueryApplicationHistoryDenial::Query(denied),
    )) = denied
    else {
        panic!("fresh metadata must refuse before any cursor or storage diagnostic")
    };
    assert_eq!(denied.kind(), worth_query_host::facade::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired);
    assert_eq!(observed.snapshot().node_probes, 0);
    assert_eq!(observed.snapshot().child_index_probes, 0);
    let lawful = document
        .redo_choices_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
            limits(1),
            None,
        )
        .unwrap();
    assert_eq!(lawful.choice(), Choice::NoChild);
    assert!(observed.snapshot().child_index_probes > 0);
    drop(observed);
    let branch = document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap()
        .endpoint()
        .product_branch();
    super::super::metadata::grant(&host, branch, "revoked", 0x9182_f230);
    let observed = document.observe_history_enrollment_for_test();
    let denied = document
        .redo_choices_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
            limits(1),
            None,
        )
        .unwrap_err();
    assert!(matches!(denied, Failure::Document(Native::Metadata(worth_query_host::facade::primary_graph::WorthQueryApplicationHistoryDenial::Authorization(_)))), "{denied:?}");
    assert_eq!(observed.snapshot().child_index_probes, 0);
    drop(observed);
    super::super::metadata::grant(&host, branch, "active", 0x9182_f231);
    document
        .refresh_context_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
            WorthQueryHistoryContextRefreshLimits {
                maximum_commits: NonZeroUsize::new(8).unwrap(),
                maximum_records: NonZeroUsize::new(256).unwrap(),
                maximum_evidence_bytes: NonZeroU64::new(8 * 1024 * 1024).unwrap(),
            },
        )
        .unwrap();
    assert_eq!(
        document
            .redo_choices_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
                limits(1),
                None
            )
            .unwrap()
            .choice(),
        Choice::NoChild
    );
    close(&host, document, 1);
    close(&host, public, 5);
}
