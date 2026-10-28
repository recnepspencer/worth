//! Actual provider calls under the distinct purpose and original Parent request.
use super::{
    retention::WorthQueryManagedIndexLifecycleDenial, selection::SelectionDenial,
    source_acquisition::AcquisitionDenial, WorthQueryManagedIndexWorkDenial,
};
use crate::domain_computation::provider_session::{
    WorthQueryManagedGraphWorkSessionStartDenial, WorthQueryManagedIndexSessionDenial,
};
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkCapacityAdmissionStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

mod build;
mod purpose;
mod select;
pub(super) use build::rebuild;
pub(super) use select::select;

pub(super) enum ProviderExecutionDenial {
    Execution(RelationalExecutionDenialCause),
    Capacity(WorthQueryGraphWorkCapacityAdmissionStop<RelationalExecutionDenialCause>),
    Identity(WorthQueryManagedGraphWorkSessionStartDenial),
    Session(WorthQueryManagedIndexSessionDenial),
    Acquisition(AcquisitionDenial),
    Lifecycle(WorthQueryManagedIndexLifecycleDenial),
    Work(WorthQueryManagedIndexWorkDenial),
    Selection(SelectionDenial),
    ForeignAssociation,
}
impl From<RelationalExecutionDenialCause> for ProviderExecutionDenial {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Execution(cause)
    }
}
