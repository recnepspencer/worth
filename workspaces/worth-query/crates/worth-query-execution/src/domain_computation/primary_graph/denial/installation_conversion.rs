//! Preserve lower-owner causes in primary graph installation refusals.
use super::super::application_contribution::WorthQueryOutputReadinessContractDenial;
use super::{
    InstallationDenialCause, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

impl From<WorthQueryOutputReadinessContractDenial> for WorthQueryPrimaryGraphInstallationDenial {
    fn from(cause: WorthQueryOutputReadinessContractDenial) -> Self {
        Self {
            kind: WorthQueryPrimaryGraphInstallationDenialKind::ReadinessContractRejected,
            subject: cause.to_string(),
            cause: Some(Box::new(InstallationDenialCause::Readiness(cause))),
        }
    }
}

impl From<worth_relational::facade::runtime::RelationalRuntimeConstructionDenial>
    for WorthQueryPrimaryGraphInstallationDenial
{
    fn from(cause: worth_relational::facade::runtime::RelationalRuntimeConstructionDenial) -> Self {
        Self::new(
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
            cause.to_string(),
        )
        .with_runtime_construction_cause(cause)
    }
}
