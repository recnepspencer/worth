use std::path::{Path, PathBuf};

use super::{
    OfflineIntegrityObservationLimits, OfflineIntegrityProtocolContext,
    OfflineIntegrityReportDestination,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineIntegrityObservationRequest {
    store_root: PathBuf,
    limits: OfflineIntegrityObservationLimits,
    report_destination: OfflineIntegrityReportDestination,
    protocol_context: OfflineIntegrityProtocolContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityObservationRequestDenial {
    EmptyStoreRoot,
}

impl OfflineIntegrityObservationRequest {
    pub fn new(
        store_root: PathBuf,
        limits: OfflineIntegrityObservationLimits,
        report_destination: OfflineIntegrityReportDestination,
        protocol_context: OfflineIntegrityProtocolContext,
    ) -> Result<Self, OfflineIntegrityObservationRequestDenial> {
        if store_root.as_os_str().is_empty() {
            return Err(OfflineIntegrityObservationRequestDenial::EmptyStoreRoot);
        }
        Ok(Self {
            store_root,
            limits,
            report_destination,
            protocol_context,
        })
    }

    pub fn store_root(&self) -> &Path {
        &self.store_root
    }

    pub const fn limits(&self) -> OfflineIntegrityObservationLimits {
        self.limits
    }

    pub const fn report_destination(&self) -> &OfflineIntegrityReportDestination {
        &self.report_destination
    }

    pub const fn protocol_context(&self) -> &OfflineIntegrityProtocolContext {
        &self.protocol_context
    }
}
