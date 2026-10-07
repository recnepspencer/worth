use std::fs::OpenOptions;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use super::report_boundary::{prove_report_destination, ProvenReportDestination};
use super::{
    encode_offline_integrity_report, OfflineIntegrityObservationRequest, OfflineIntegrityReport,
    OfflineIntegrityReportBoundaryDenial, OfflineIntegrityReportWireDenial,
};
use crate::{
    encode_offline_selected_integrity_observation, PhysicalIntegrityComparisonDenial,
    PhysicalIntegrityComparisonLimits,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityReportEmissionDenial {
    Boundary(OfflineIntegrityReportBoundaryDenial),
    Wire(OfflineIntegrityReportWireDenial),
    SelectedWire(PhysicalIntegrityComparisonDenial),
    ElapsedBoundExceeded,
    Io,
}

pub fn emit_offline_integrity_report(
    request: &OfflineIntegrityObservationRequest,
    report: &OfflineIntegrityReport,
) -> Result<(), OfflineIntegrityReportEmissionDenial> {
    emit_with(request, || {
        encode_offline_integrity_report(report).map_err(OfflineIntegrityReportEmissionDenial::Wire)
    })
}

pub fn emit_offline_selected_integrity_report(
    request: &OfflineIntegrityObservationRequest,
    report: &OfflineIntegrityReport,
) -> Result<(), OfflineIntegrityReportEmissionDenial> {
    let limits = PhysicalIntegrityComparisonLimits::new(
        request.limits().maximum_report_bytes(),
        request.limits().maximum_entries(),
        request.limits().maximum_report_bytes(),
    )
    .map_err(|_| {
        OfflineIntegrityReportEmissionDenial::SelectedWire(
            PhysicalIntegrityComparisonDenial::InvalidObservation,
        )
    })?;
    emit_with(request, || {
        encode_offline_selected_integrity_observation(report, limits)
            .map_err(OfflineIntegrityReportEmissionDenial::SelectedWire)
    })
}

fn emit_with(
    request: &OfflineIntegrityObservationRequest,
    encode: impl FnOnce() -> Result<String, OfflineIntegrityReportEmissionDenial>,
) -> Result<(), OfflineIntegrityReportEmissionDenial> {
    let started = Instant::now();
    let (_, destination) = prove_report_destination(
        request.store_root(),
        request.report_destination(),
        request.limits(),
        started,
    )
    .map_err(OfflineIntegrityReportEmissionDenial::Boundary)?;
    let wire = encode()?;
    ensure_elapsed(request, started)?;
    match destination {
        ProvenReportDestination::StandardOutput => {
            write_with_elapsed_bound(io::stdout().lock(), wire.as_bytes(), request, started)
        }
        ProvenReportDestination::File {
            path,
            _parent_guard,
        } => {
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|_| OfflineIntegrityReportEmissionDenial::Io)?;
            write_with_elapsed_bound(&mut output, wire.as_bytes(), request, started)
        }
    }
}

fn write_with_elapsed_bound(
    mut output: impl Write,
    mut bytes: &[u8],
    request: &OfflineIntegrityObservationRequest,
    started: Instant,
) -> Result<(), OfflineIntegrityReportEmissionDenial> {
    while !bytes.is_empty() {
        ensure_elapsed(request, started)?;
        let written = output
            .write(bytes)
            .map_err(|_| OfflineIntegrityReportEmissionDenial::Io)?;
        if written == 0 {
            return Err(OfflineIntegrityReportEmissionDenial::Io);
        }
        bytes = &bytes[written..];
    }
    ensure_elapsed(request, started)
}

fn ensure_elapsed(
    request: &OfflineIntegrityObservationRequest,
    started: Instant,
) -> Result<(), OfflineIntegrityReportEmissionDenial> {
    (started.elapsed() < Duration::from_millis(request.limits().maximum_elapsed_milliseconds()))
        .then_some(())
        .ok_or(OfflineIntegrityReportEmissionDenial::ElapsedBoundExceeded)
}
