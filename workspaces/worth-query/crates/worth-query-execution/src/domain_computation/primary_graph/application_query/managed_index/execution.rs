//! Actual provider calls under the distinct purpose and original Parent request.
use {super::retention::WorthQueryManagedIndexLifecycleDenial, super::selection::SelectionDenial, super::source_acquisition::AcquisitionDenial, crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial};
use crate::domain_computation::provider_session::{
    WorthQueryManagedGraphWorkSessionStartDenial, WorthQueryManagedIndexSessionDenial,
};
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkCapacityAdmissionStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

mod build;
mod maintained_build;
pub(super) mod purpose;
mod select;
pub(super) use build::rebuild;
pub(super) use select::select;

pub(super) enum ProviderExecutionDenial {
    SourceVerification(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial),
    Execution(RelationalExecutionDenialCause),
    Capacity(WorthQueryGraphWorkCapacityAdmissionStop<RelationalExecutionDenialCause>),
    Identity(WorthQueryManagedGraphWorkSessionStartDenial),
    Session(WorthQueryManagedIndexSessionDenial),
    Acquisition(AcquisitionDenial),
    Lifecycle(WorthQueryManagedIndexLifecycleDenial),
    Work(WorthQueryManagedIndexWorkDenial),
    Selection(SelectionDenial),
    SourceRead(super::super::output_source_observation::SourceQueryObservationStop),
    NativeSource(worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial),
    SourceCompletion(super::source_completion::SourceDenial),
    GraphRead(crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial),
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    NativeAdmission(worth_relational::facade::mvcc::CompanionPreflightStop),
    ForeignAssociation,
}
impl From<super::super::output_source_observation::SourceQueryObservationStop>
    for ProviderExecutionDenial
{
    fn from(cause: super::super::output_source_observation::SourceQueryObservationStop) -> Self {
        Self::SourceRead(cause)
    }
}
impl From<RelationalExecutionDenialCause> for ProviderExecutionDenial {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Execution(cause)
    }
}
