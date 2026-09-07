use std::time::Instant;

use super::namespace_identity_walk::observe_namespace_identity;
use super::report_boundary::prove_report_destination;
use super::root_protocol_walk::observe_root_protocol;
use super::{
    BoundedMediaWalk, OfflineIntegrityObservationRequest, OfflineIntegrityReport,
    OfflineIntegrityReportBoundaryDenial, OfflineIntegrityReportWireDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityObservationDenial {
    ReportBoundary(OfflineIntegrityReportBoundaryDenial),
    NamespaceDirectoryUnavailable,
    RecordDirectoryUnavailable,
    RootDirectoryUnavailable,
    ReportWire(OfflineIntegrityReportWireDenial),
}

pub fn observe_store(
    request: &OfflineIntegrityObservationRequest,
) -> Result<OfflineIntegrityReport, OfflineIntegrityObservationDenial> {
    let started = Instant::now();
    let (store_root, _) = prove_report_destination(
        request.store_root(),
        request.report_destination(),
        request.limits(),
        started,
    )
    .map_err(OfflineIntegrityObservationDenial::ReportBoundary)?;
    let mut walk = BoundedMediaWalk::new(request.limits(), store_root.clone(), started);
    let namespace = observe_namespace_identity(&store_root, &mut walk)
        .map_err(|_| OfflineIntegrityObservationDenial::NamespaceDirectoryUnavailable)?;
    let root_observations =
        observe_root_protocol(&store_root, namespace.expected_store_identity, &mut walk)?;
    let store_identity = namespace
        .expected_store_identity
        .map(|identity| hex_bytes(&identity).into_boxed_str());
    let mut observations = namespace.observations;
    observations.extend(root_observations.artifacts);
    observations.extend(super::record_walk::observe_records(
        &store_root,
        namespace.expected_store_identity,
        &root_observations.roots,
        &mut walk,
    ));
    observations.extend(super::journal_walk::observe_journals(
        &store_root,
        namespace.expected_store_identity,
        &mut walk,
    ));
    let residue = super::namespace_inventory::observe_namespace_residue(
        &store_root,
        &observations,
        &mut walk,
    );
    observations.extend(residue);
    observations.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));
    let (counters, completeness) = walk.finish();
    let mut report = OfflineIntegrityReport::new(
        request.protocol_context().clone(),
        store_identity,
        request.limits(),
        counters,
        completeness,
        observations,
    );
    super::report_wire::stabilize_report_bytes(&mut report)
        .map_err(OfflineIntegrityObservationDenial::ReportWire)?;
    Ok(report)
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}
