//! Entity-only refresh cannot silently bless movement inside an owned graph.
use super::*;

#[test]
fn declared_ownership_refuses_the_entity_only_context_refresh_contract() {
    let host = world::install();
    let mut document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let limits = WorthQueryHistoryContextRefreshLimits {
        maximum_commits: std::num::NonZeroUsize::new(16).unwrap(),
        maximum_records: std::num::NonZeroUsize::new(128).unwrap(),
        maximum_evidence_bytes: std::num::NonZeroU64::new(1_048_576).unwrap(),
    };
    assert!(matches!(
        document.refresh_context(&request, limits),
        Err(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::UnsupportedContextRefresh
        ))
    ));
    assert_eq!(document.selection(&request).unwrap(), before);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
