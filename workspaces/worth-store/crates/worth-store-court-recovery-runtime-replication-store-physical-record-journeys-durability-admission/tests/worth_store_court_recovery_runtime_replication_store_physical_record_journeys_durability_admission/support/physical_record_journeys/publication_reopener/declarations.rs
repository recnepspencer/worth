pub(crate) use {std::io::Write, std::path::Path};

pub(crate) use {worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanDenial, worth_store::physical_runtime::RecordScanRequest};

pub(crate) fn run(root: &Path) {
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(root);
    let residue = serving.observed_non_authoritative_residue();
    assert!(!serving.physical_recovery_evidence_damaged());
    let recovery_count = serving.physical_recovery_obligations().len();
    let fenced = residue || recovery_count != 0;
    let records = if fenced {
        let scan = serving.records().expect("read protection admission").scan(
            worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(17).unwrap()),
        );
        assert!(matches!(
            scan,
            Err(error) if error.denial() == worth_store::physical_runtime::RecordScanDenial::ServingRequiresInspection
        ));
        0
    } else {
        worth_store_testkit_store::physical_record_journeys::scan_journeys::collect_scan(&serving, 17, 64_000).len()
    };
    worth_store_testkit_store::scenario_process_evidence::emit_process("fresh-reopener", &serving);
    println!(
        "C5_PUBLICATION_REOPEN {} {} {} {} {}",
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        records,
        residue,
        fenced,
        recovery_count,
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}