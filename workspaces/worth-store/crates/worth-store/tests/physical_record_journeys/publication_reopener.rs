use std::{io::Write, path::Path};

use worth_store::physical_runtime::{RecordCountLimit, RecordScanDenial, RecordScanRequest};

pub(super) fn run(root: &Path) {
    let serving = super::serving_from_open(root);
    let residue = serving.observed_non_authoritative_residue();
    assert!(!serving.physical_recovery_evidence_damaged());
    let recovery_count = serving.physical_recovery_obligations().len();
    let fenced = residue || recovery_count != 0;
    let records = if fenced {
        let scan = serving.records().expect("read protection admission").scan(
            RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(17).unwrap()),
        );
        assert!(matches!(
            scan,
            Err(error) if error.denial() == RecordScanDenial::ServingRequiresInspection
        ));
        0
    } else {
        super::scan_journeys::collect_scan(&serving, 17, 64_000).len()
    };
    super::scenario_evidence::emit_process("fresh-reopener", &serving);
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
