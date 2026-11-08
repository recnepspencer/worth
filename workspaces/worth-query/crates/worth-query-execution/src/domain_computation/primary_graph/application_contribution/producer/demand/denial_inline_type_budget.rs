//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryOutputDemandDenial>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/denial.rs", "WorthQueryOutputDemandDenial", 128 * 1024);
        owner.field::<Box<WorthQueryOutputDemandDenialKind>>("kind");
        owner.field::<Option<Box<DemandDenialEvidence>>>("evidence");
        owner.field::<Option<cleanup_custody::DemandCleanupCustody>>("cleanup");
        owner.field::<std::borrow::Cow<'static, str>>("subject");
        owner.field::<Option<Box<producer_value::ProducerDomainDenial>>>("domain");
        owner.field::<Option<&'static str>>("readmission_failure");
        owner.field::<WorthQueryOutputDemandRecoveryPosture>("recovery_posture");
    }
    super::kind::inline_type_budget::collect(census);
}
