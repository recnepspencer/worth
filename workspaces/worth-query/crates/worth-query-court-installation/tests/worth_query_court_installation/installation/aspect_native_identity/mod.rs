pub(crate) mod model;

#[test]
fn declaration_order_converges_to_the_same_installed_identity() {
    let forward = model::declaration::<8, 9, false>().unwrap();
    let reverse = model::declaration::<8, 9, true>().unwrap();
    assert_eq!(forward.identity(), reverse.identity());
    let index = model::index(&forward);
    let forward = index
        .bind_application_schema(forward)
        .unwrap()
        .installed_operation(model::operation())
        .unwrap();
    let reverse = index
        .bind_application_schema(reverse)
        .unwrap()
        .installed_operation(model::operation())
        .unwrap();
    assert_eq!(
        forward.graph_obligations().identity(),
        reverse.graph_obligations().identity()
    );
}

#[test]
fn aspect_native_source_and_realization_operation_installs() {
    let installed = model::install::<8, 9>().expect("consumer-shaped apply-preserve must install");
    let work = installed
        .graph_obligations()
        .installation_evidence()
        .canonical_work();
    println!(
        "aspect-native records=8 fields=9 entries={} bytes={}",
        work.canonical_entries(),
        work.canonical_encoded_bytes()
    );
}

#[test]
fn truly_excessive_distinct_facts_refuse_with_requested_and_maximum_counts() {
    use worth_query_installation::facade::CanonicalDigestDerivationDenial;
    use worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenialKind;
    let denial = model::install::<18, 16>()
        .err()
        .expect("identity limit still applies");
    println!("{denial}");
    assert_eq!(denial.operation(), "apply-preserve");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOperationInstallationDenialKind::CanonicalEntryBudgetExceeded
    );
    assert!(
        matches!(denial.canonical_denial(), Some(CanonicalDigestDerivationDenial::EntryLimitExceeded { actual, maximum }) if *maximum == 4096 && *actual > *maximum)
    );
}
