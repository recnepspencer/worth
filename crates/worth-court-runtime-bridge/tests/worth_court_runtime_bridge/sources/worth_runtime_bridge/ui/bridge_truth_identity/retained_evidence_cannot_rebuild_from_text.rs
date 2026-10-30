use worth_runtime_bridge::facade::BridgeIdentityEvidence;

fn main() {
    let _evidence = BridgeIdentityEvidence::from_query_evidence_identity(
        "query-evidence:scope",
        "query-evidence:digest",
    );
}
