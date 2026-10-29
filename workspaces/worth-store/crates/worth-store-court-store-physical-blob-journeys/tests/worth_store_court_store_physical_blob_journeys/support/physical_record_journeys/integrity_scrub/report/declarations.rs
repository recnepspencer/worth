pub(crate) use {worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress as Progress, worth_store_testkit_store::integrity_scrub::assert_released, crate::physical_record_journeys::integrity_scrub::report, worth_store_testkit_store::integrity_scrub::request, worth_store_testkit_store::integrity_scrub::target};
pub(crate) use {worth_store::integrity_observation::PhysicalIntegrityRuntimeReportContext as Context, worth_store::integrity_observation::PhysicalIntegrityRuntimeReportDenial as ReportDenial};

pub(crate) fn capture(
    handle: &mut worth_store::physical_runtime::ManagedPhysicalIntegrityScrubHandle,
    run: &str,
) -> String {
    // Output storage belongs to this caller, not to the Store's source budget.
    let mut bytes = Vec::new();
    let count = handle
        .write_observation_report(
            worth_store::integrity_observation::PhysicalIntegrityRuntimeReportContext::new(run, "runtime-report").unwrap(),
            16_384,
            &mut bytes,
        )
        .unwrap();
    assert_eq!(count, bytes.len() as u64);
    String::from_utf8(bytes).unwrap()
}