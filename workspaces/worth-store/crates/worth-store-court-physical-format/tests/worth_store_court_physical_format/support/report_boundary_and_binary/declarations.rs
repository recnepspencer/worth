pub(crate) use std::path::{Component, Path, PathBuf};
pub(crate) use std::process::Command;

pub(crate) use worth_store_offline_integrity_observer::{
    emit_offline_integrity_report, encode_offline_integrity_report, observe_store,
    OfflineIntegrityObservationDenial, OfflineIntegrityObservationLimits,
    OfflineIntegrityObservationRequest, OfflineIntegrityProtocolContext,
    OfflineIntegrityReportBoundaryDenial, OfflineIntegrityReportDestination,
};

pub(crate) use crate::{
    store_offline_integrity_observer_support::artifact_path,
    store_offline_integrity_observer_support::request,
    store_offline_integrity_observer_support::Target, support::fixtures::clean_store,
    support::fixtures::StoreFixture, support::json::parse_json,
};

pub(crate) fn assert_wire_posture(wire: &str, expected: &str) {
    let parsed = parse_json(wire);
    assert_eq!(
        parsed.field("protocol").string(),
        "store.physical.integrity-observation"
    );
    assert!(parsed
        .field("artifacts")
        .array()
        .iter()
        .any(|artifact| artifact.field("outcome").field("posture").string() == expected));
    assert_eq!(
        parsed.field("consumed").field("report_bytes").number(),
        wire.len() as u64
    );
}

pub(crate) fn assert_inside_denial(fixture: &StoreFixture, destination: PathBuf) {
    let request = request_with_destination(fixture, destination);
    assert_eq!(
        observe_store(&request),
        Err(worth_store_offline_integrity_observer::OfflineIntegrityObservationDenial::ReportBoundary(
            worth_store_offline_integrity_observer::OfflineIntegrityReportBoundaryDenial::DestinationInsideStoreRoot,
        ))
    );
}

pub(crate) fn request_with_destination(
    fixture: &StoreFixture,
    destination: PathBuf,
) -> OfflineIntegrityObservationRequest {
    let template = request(fixture);
    worth_store_offline_integrity_observer::OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        template.limits(),
        worth_store_offline_integrity_observer::OfflineIntegrityReportDestination::file(
            destination,
        )
        .unwrap(),
        worth_store_offline_integrity_observer::OfflineIntegrityProtocolContext::new(
            "fixture-observer",
            "process-1",
            "run-1",
            "scenario-1",
        )
        .unwrap(),
    )
    .unwrap()
}

pub(crate) fn bounded_request(
    fixture: &StoreFixture,
    limits: OfflineIntegrityObservationLimits,
) -> OfflineIntegrityObservationRequest {
    let template = request(fixture);
    worth_store_offline_integrity_observer::OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        limits,
        worth_store_offline_integrity_observer::OfflineIntegrityReportDestination::standard_output(
        ),
        template.protocol_context().clone(),
    )
    .unwrap()
}

pub(crate) fn relative_from_current_directory(target: &Path) -> PathBuf {
    let current = std::env::current_dir().unwrap();
    let current_components: Vec<_> = current.components().collect();
    let target_components: Vec<_> = target.components().collect();
    let shared = current_components
        .iter()
        .zip(&target_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = std::path::PathBuf::new();
    for component in &current_components[shared..] {
        if matches!(component, std::path::Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &target_components[shared..] {
        relative.push(component.as_os_str());
    }
    relative
}

#[cfg(unix)]
pub(crate) fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub(crate) fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}
