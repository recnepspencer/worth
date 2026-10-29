use worth_server::{
    WorthServerAbuseBudgetReceipt, WorthServerBinaryCounterSet, WorthServerCompatHttpRouteFamily,
    WorthServerExternalCounterSet, WorthServerTransferByteClass,
    WorthServerTransferCleanupEvidence, WorthServerTransferCleanupReason,
};

pub(crate) fn assert_budget_receipt(
    receipt: &WorthServerAbuseBudgetReceipt,
    expected_route_family: WorthServerCompatHttpRouteFamily,
    expected_byte_class: WorthServerTransferByteClass,
    expected_denial_fragment: Option<&str>,
) {
    assert_eq!(receipt.route_family(), expected_route_family);
    assert_eq!(receipt.byte_class(), expected_byte_class);
    match expected_denial_fragment {
        Some(fragment) => assert!(
            receipt
                .denial()
                .expect("expected denial detail")
                .contains(fragment),
            "expected denial detail to contain `{fragment}`"
        ),
        None => assert!(receipt.denial().is_none()),
    }
}

pub(crate) fn assert_budget_scope(
    receipt: &WorthServerAbuseBudgetReceipt,
    expected_tenant_id: &str,
    expected_workspace_id: &str,
    expected_branch_id: &str,
) {
    let expected_workspace_digest =
        format!("worth-server-workspace-target-v1:{expected_tenant_id}:{expected_workspace_id}");
    let expected_branch_digest =
        format!("worth-server-branch-target-v1:branch:{expected_branch_id}");
    assert_eq!(receipt.tenant_id(), expected_tenant_id);
    assert_eq!(receipt.workspace_digest(), expected_workspace_digest);
    assert_eq!(receipt.branch_digest(), expected_branch_digest);
    let canonical_digest = receipt.canonical_digest();
    assert!(canonical_digest.contains(&format!("tenant={expected_tenant_id}")));
    assert!(canonical_digest.contains(&format!("workspace={expected_workspace_digest}")));
    assert!(canonical_digest.contains(&format!("branch={expected_branch_digest}")));
}

pub(crate) fn assert_external_counter(
    counters: &WorthServerExternalCounterSet,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        counters.counter(name),
        Some(expected),
        "external counter `{name}`"
    );
}

pub(crate) fn assert_binary_counter(
    counters: &WorthServerBinaryCounterSet,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        counters.counter(name),
        Some(expected),
        "binary counter `{name}`"
    );
}

pub(crate) fn assert_cleanup_evidence(
    evidence: &WorthServerTransferCleanupEvidence,
    expected_route_family: WorthServerCompatHttpRouteFamily,
    expected_byte_class: WorthServerTransferByteClass,
    expected_reason: WorthServerTransferCleanupReason,
) {
    assert_eq!(evidence.route_family(), expected_route_family);
    assert_eq!(evidence.byte_class(), expected_byte_class);
    assert_eq!(evidence.reason(), expected_reason);
    assert!(evidence.attachment_bundle().provenance().is_some());
    assert!(evidence.attachment_bundle().receipt().is_some());
}

pub(crate) fn assert_cleanup_scope(
    evidence: &WorthServerTransferCleanupEvidence,
    expected_tenant_id: &str,
    expected_workspace_id: &str,
    expected_branch_id: &str,
) {
    let expected_workspace_digest =
        format!("worth-server-workspace-target-v1:{expected_tenant_id}:{expected_workspace_id}");
    let expected_branch_digest =
        format!("worth-server-branch-target-v1:branch:{expected_branch_id}");
    assert_eq!(evidence.tenant_id(), expected_tenant_id);
    assert_eq!(evidence.workspace_digest(), expected_workspace_digest);
    assert_eq!(evidence.branch_digest(), expected_branch_digest);
    let canonical_digest = evidence.canonical_digest();
    assert!(canonical_digest.contains(&format!("tenant={expected_tenant_id}")));
    assert!(canonical_digest.contains(&format!("workspace={expected_workspace_digest}")));
    assert!(canonical_digest.contains(&format!("branch={expected_branch_digest}")));
}
