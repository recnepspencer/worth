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

impl From<worth_query_execution_layout::facade::WorthQuerySchemaLayoutDenial>
    for WorthQueryPrimaryGraphInstallationDenial
{
    fn from(cause: worth_query_execution_layout::facade::WorthQuerySchemaLayoutDenial) -> Self {
        use worth_query_execution_layout::facade::WorthQuerySchemaLayoutDenialKind as LayoutKind;
        let (layout_kind, subject, content_cause) = cause.into_parts();
        let kind = match layout_kind {
            LayoutKind::ContentAdmissionRejected => {
                WorthQueryPrimaryGraphInstallationDenialKind::ContentAdmissionRejected
            }
            LayoutKind::InvalidSchemaMember => {
                WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
            }
            LayoutKind::RelationalSchemaRejected => {
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected
            }
            LayoutKind::UnsupportedManagedSource => {
                WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedManagedSource
            }
            LayoutKind::SourceProgramIdentityExhausted => {
                WorthQueryPrimaryGraphInstallationDenialKind::SourceProgramIdentityExhausted
            }
            LayoutKind::OrderedPredecessorMembershipRequired => {
                WorthQueryPrimaryGraphInstallationDenialKind::OrderedPredecessorMembershipRequired
            }
            LayoutKind::OrderedPredecessorMustBeToOne => {
                WorthQueryPrimaryGraphInstallationDenialKind::OrderedPredecessorMustBeToOne
            }
        };
        let denial = Self::new(kind, subject);
        match content_cause {
            Some(content) => denial.with_content_cause(content),
            None => denial,
        }
    }
}
