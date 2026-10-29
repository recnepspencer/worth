pub(crate) use worth_query_worlds::facade::certification::application_graph::producer_predicate_checkpoint::ordinary_source::terminal_unavailable::*;

#[test]
fn fixed_leaf_domain_refusal_closes_unavailable_without_reexecution() {
    let source_before = (decision_count(0), candidate_count(0));
    let assessment_before = assessment_counts(0);
    let mut host = install();
    let terminal = terminal_owner(&host);
    assert_eq!(host.retained_source_custody_count_for_test(), 1);
    assert_eq!(
        host.close_conditional_runtime().unwrap_err().kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained
    );
    let terminal = refuse_foreign_and_cancelled(&host, terminal);
    assert!(terminal.output_unavailable().is_none());
    let closed = close_original(&host, terminal);
    assert!(closed.output_unavailable().is_some());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    assert_eq!(
        request
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        0,
        "source is independently saved despite no assessment output"
    );
    assert_eq!(decision_count(0), source_before.0 + 1);
    assert_eq!(candidate_count(0), source_before.1 + 1);
    assert_eq!(
        assessment_counts(0),
        (assessment_before.0 + 1, assessment_before.1),
        "exactly one real producer decision, zero output candidates or replay"
    );
    assert_eq!(host.retained_source_custody_count_for_test(), 0);
    assert_eq!(host.prepared_required_output_source_count_for_test(), 0);
    assert_eq!(host.active_application_attempts_for_test(), 0);
    assert_eq!(host.world_active_publication_attempts_for_test(), 0);
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    host.close_conditional_runtime()
        .expect("lawful operation close released its authentic runtime keepalive");
}
