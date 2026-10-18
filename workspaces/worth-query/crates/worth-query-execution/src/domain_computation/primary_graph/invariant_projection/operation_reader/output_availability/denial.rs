//! Original native owner stops from a non-driving output observation.
use crate::domain_computation::primary_graph::{
    application_query::SourceQueryObservationStop,
    invariant_projection::ConsumedOutputVerificationStop,
    output_lineage::invalidation::SettlementRegistrationStop, WorthQueryCurrentOutputDenial,
    WorthQueryInvariantDecisionPlanDenial, WorthQueryOutputDemandDenial,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputAvailabilityDenial {
    cause: Box<Cause>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Cause {
    ContextUnavailable,
    SourceMismatch,
    Decision(WorthQueryInvariantDecisionPlanDenial),
    SourceQuery(SourceQueryObservationStop),
    Producer(WorthQueryOutputDemandDenial),
    Verification(ConsumedOutputVerificationStop),
    Registration(SettlementRegistrationStop),
    Current(WorthQueryCurrentOutputDenial),
}
impl From<Cause> for WorthQueryOutputAvailabilityDenial {
    fn from(cause: Cause) -> Self {
        Self {
            cause: Box::new(cause),
        }
    }
}
impl std::fmt::Display for WorthQueryOutputAvailabilityDenial {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "output availability: {:?}", self.cause)
    }
}
impl std::error::Error for WorthQueryOutputAvailabilityDenial {}

impl WorthQueryOutputAvailabilityDenial {
    /// The original source Query admission refusal, if that phase stopped.
    pub fn query_admission_denial(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial>
    {
        match self.cause.as_ref() {
            Cause::SourceQuery(SourceQueryObservationStop::Admission(stop))
            | Cause::SourceQuery(SourceQueryObservationStop::Preparation { cause: stop, .. }) => {
                Some(stop)
            }
            _ => None,
        }
    }
    /// The original admitted Query read refusal, if that phase stopped.
    pub fn query_execution_denial(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenial> {
        match self.cause.as_ref() { Cause::SourceQuery(SourceQueryObservationStop::Read(crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop::Execution(stop))) => Some(stop), _ => None }
    }
    pub fn current_output_denial(&self) -> Option<&WorthQueryCurrentOutputDenial> {
        match self.cause.as_ref() {
            Cause::Current(stop) => Some(stop),
            _ => None,
        }
    }
}

mod demand_stop;

impl worth_execution::ChargedBytes for WorthQueryOutputAvailabilityDenial {
    fn additional_charged_bytes(&self) -> u64 {
        use SourceQueryObservationStop as Query;
        let payload=match self.cause.as_ref(){
 Cause::Decision(stop)=>stop.additional_charged_bytes(),
 Cause::Producer(stop)=>stop.additional_charged_bytes(),
 Cause::Current(stop)=>stop.additional_charged_bytes(),
 Cause::SourceQuery(Query::Principal(stop))=>stop.additional_charged_bytes(),
 Cause::SourceQuery(Query::Scope(stop))=>stop.additional_charged_bytes(),
 Cause::SourceQuery(Query::Admission(stop)) | Cause::SourceQuery(Query::Preparation { cause: stop, .. })=>stop.additional_charged_bytes(),
 Cause::SourceQuery(Query::SourceFacts(stop))=>stop.additional_charged_bytes(),
 Cause::SourceQuery(Query::Read(crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop::Execution(stop)))=>stop.additional_charged_bytes(),
 // Installation diagnostics remain owned by Installation, which does not yet
 // expose a resident-error quote. They refuse retained error storage.
 Cause::SourceQuery(Query::Installation(worth_query_installation::facade::WorthQueryInstalledQueryBindingAdmissionStop::Installation(_)))=>u64::MAX,
 Cause::ContextUnavailable|Cause::SourceMismatch|Cause::Verification(_)|Cause::Registration(_)|Cause::SourceQuery(_)=>0,
 };
        (std::mem::size_of::<Cause>() as u64).saturating_add(payload)
    }
}
