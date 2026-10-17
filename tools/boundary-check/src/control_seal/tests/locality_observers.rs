//! The private locality court has one exact observation source, no neighbors.
use super::*;

#[test]
fn private_locality_observer_caller_is_exact_and_neighbors_are_refused() {
    let owner = "workspaces/worth-query/crates/worth-query-execution/";
    let caller = "crates/worthy-studio-application/src/tests/locality/observations.rs";
    let names = [
        "fact_verification_on_this_thread_for_test",
        "WorthQueryFactVerificationForTest",
        "native_marking_on_this_thread_for_test",
        "WorthQueryNativeMarkingForTest",
        "WorthQueryMarkedOutputForTest",
        "WorthQueryNativeMarkingPrecision",
        "WorthQueryNativeMarkingWork",
    ];
    // Only *_for_test/*ForTest are control doors; precision/work are ordinary
    // descriptive types and require no additional export or permission.
    for name in names
        .into_iter()
        .filter(|name| name.ends_with("_for_test") || name.ends_with("ForTest"))
    {
        let doors = BTreeMap::from([(name.to_owned(), BTreeSet::from([owner.to_owned()]))]);
        let source = format!("fn observe() {{ let _ = {name}; }}");
        assert!(
            references(caller, &source, &doors).is_empty(),
            "{name}: exact caller"
        );
        for neighbor in [
            "crates/worthy-studio-application/src/tests/locality/observations.neighbor.rs",
            "crates/worthy-studio-application/src/tests/locality/neighbor.rs",
            "crates/worthy-studio-application/src/locality/observations.rs",
        ] {
            let refused = references(neighbor, &source, &doors);
            assert_eq!(refused.len(), 1, "{name}: {neighbor}");
            assert_eq!(refused[0].code(), DiagnosticCode::Bc7012ControlSeal);
        }
    }
}
