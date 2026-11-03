use worth_query_execution::facade::domain_computation::WorthQueryGraphWorkManagedRunIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryGraphWorkSessionIdentity;

#[test]
fn session_identity_is_fixed_width_and_needs_no_digest_text() {
    let identity = WorthQueryGraphWorkSessionIdentity::mint().unwrap();
    assert_eq!(std::mem::size_of_val(&identity), std::mem::size_of::<u64>());
    assert!(identity.as_u64() > 0);
}

#[test]
fn managed_run_identity_is_distinct_and_fixed_width() {
    let run = WorthQueryGraphWorkManagedRunIdentity::mint().unwrap();
    let session = WorthQueryGraphWorkSessionIdentity::mint().unwrap();

    assert_eq!(std::mem::size_of_val(&run), std::mem::size_of::<u64>());
    assert!(run.as_u64() > 0);
    assert!(session.as_u64() > 0);
}
