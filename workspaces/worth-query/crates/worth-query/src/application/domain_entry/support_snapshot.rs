use crate::application::WorthQueryDomainOperatingRequirement;
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryCapabilityStatus, WorthQuerySupportMatrix,
    WorthQuerySupportReport, WorthQuerySupportSectionPosture,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryRuntimeBackendPosture, WorthQueryRuntimeFamilySupportStatus,
    WorthQueryRuntimePublicApiContract, WorthQueryRuntimePublicSupportMatrix,
    WorthQueryRuntimeSupportProfile,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainEntrySupportSnapshot {
    report: WorthQuerySupportReport,
    runtime_support_matrix: WorthQueryRuntimePublicSupportMatrix,
    snapshot_digest: String,
}

impl WorthQueryDomainEntrySupportSnapshot {
    pub(crate) fn from_support_report(
        report: WorthQuerySupportReport,
    ) -> Result<Self, IdentityDerivationDenial> {
        let runtime_support_matrix =
            WorthQueryRuntimePublicSupportMatrix::from_public_api_contract(
                &WorthQueryRuntimePublicApiContract::from_support_profile(
                    &WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
                        .with_posture(WorthQueryRuntimeBackendPosture::Primary),
                )?,
            )?;
        let snapshot_digest = hash_parts(&[
            format!("report:{}", report.report_digest()),
            format!(
                "runtime_support:{}",
                runtime_support_matrix
                    .matrix_digest()
                    .terminal_projection_for_reporting()
            ),
        ]);
        Ok(Self {
            report,
            runtime_support_matrix,
            snapshot_digest,
        })
    }

    pub fn support_matrix(&self) -> &WorthQuerySupportMatrix {
        self.report.support_matrix()
    }

    pub fn admitted_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        self.report.admitted_capability_families()
    }

    pub fn deferred_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        self.report.deferred_capability_families()
    }

    pub fn unsupported_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        self.report.unsupported_capability_families()
    }

    pub fn section_postures(&self) -> &[WorthQuerySupportSectionPosture] {
        self.report.section_postures()
    }

    pub fn runtime_support_matrix(&self) -> &WorthQueryRuntimePublicSupportMatrix {
        &self.runtime_support_matrix
    }

    pub fn validated_config_digest(&self) -> &str {
        self.report.validated_config_digest()
    }

    pub fn snapshot_digest(&self) -> &str {
        &self.snapshot_digest
    }

    pub fn capability_status(
        &self,
        family: WorthQueryCapabilityFamily,
    ) -> Option<WorthQueryCapabilityStatus> {
        self.support_matrix()
            .descriptor(family)
            .map(|descriptor| descriptor.status())
    }

    pub fn operating_requirement_status(
        &self,
        requirement: WorthQueryDomainOperatingRequirement,
    ) -> Option<WorthQueryRuntimeFamilySupportStatus> {
        self.runtime_support_matrix()
            .row_for_family(requirement.runtime_facade_family())
            .map(|row| row.status())
    }
}
