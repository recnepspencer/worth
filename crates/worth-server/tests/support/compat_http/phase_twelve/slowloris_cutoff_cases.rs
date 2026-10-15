use worth_proof::TransitionOutcome;
use worth_server::{WorthServerCompatHttpRouteFamily, WorthServerTransferByteClass};

use {crate::shared_support_compat_http_phase_twelve_assertions::{
        assert_binary_counter, assert_budget_receipt, assert_budget_scope,
    }, crate::shared_support_compat_http_phase_twelve_runtime::{build_phase_twelve_server, drip_fed_upload, upload_input}};

#[test]
fn compat_http_phase_twelve_drip_feed_cutoffs_emit_typed_budget_denials_without_truth_drift() {
    let server = build_phase_twelve_server();
    let denial = match server
        .compat_http()
        .upload(upload_input(&server, drip_fed_upload()))
    {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected hostile drip-feed upload denial, got {other:?}"),
    };
    let receipt = denial
        .abuse_budget_receipt()
        .expect("drip-feed denial should retain abuse budget evidence");
    assert_budget_receipt(
        receipt,
        WorthServerCompatHttpRouteFamily::Upload,
        WorthServerTransferByteClass::BinaryWire,
        Some("chunk pacing cap"),
    );
    assert_budget_scope(receipt, "tenant-a", "workspace-42", "branch-9");
    let counters = receipt
        .binary_counters()
        .expect("binary ingress cutoff should expose binary counters");
    assert_binary_counter(counters, "compat_http.abuse.denied", 1);
    assert_binary_counter(counters, "compat_http.abuse.admitted", 0);
    assert_binary_counter(counters, "compat_http.transfer.slowloris_cutoffs", 1);
    assert_binary_counter(counters, "compat_http.transfer.semantic_truth_drift", 0);
}
