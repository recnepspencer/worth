//! Current authorization and retry ownership for a producer's nested source reads.
use super::*;
use crate::domain_computation::primary_graph::{
    provider::WorthQueryApplicationBranchCommitLane, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};

pub mod ordinary;

impl<'a, S: ApplicationSchema> SourceQueryReaderAccess<'a, S> {
    pub fn ordinary(
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        shared: &'a SharedSelectedProductOperation<'a, S>,
        principal: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            application,
            shared,
            principal,
            request,
            current: None,
            predecessor: None,
            original: None,
            observation_refused: Default::default(),
            applied_participants: None,
            applied_source_publication: None,
            applied_managed_sources: None,
            read_authorization: None,
            refused: Default::default(),
        }
    }
    pub fn record_control_refusal(
        &self,
        stop: SourceQueryObservationStop,
    ) -> SourceQueryObservationStop {
        if self.current.is_some() && stop.is_control_refusal() {
            return self
                .refused
                .borrow_mut()
                .get_or_insert_with(|| stop.clone())
                .clone();
        }
        stop
    }

    /// A handler cannot swallow a control refusal or turn it into domain denial.
    pub fn take_control_failure(&self) -> Option<WorthQueryOutputDemandDenial> {
        if let Some(cause) = self.observation_refused.borrow_mut().take() {
            if let super::predecessor::Denial::Control(stop) = cause {
                return Some(crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(*stop).demand_stop());
            }
            let subject = format!("producer observation read: {cause}");
            let kind = match &cause {
                super::predecessor::Denial::Authorization(cause) => {
                    WorthQueryOutputDemandDenialKind::of_request_authorization(cause.kind())
                }
                _ => WorthQueryOutputDemandDenialKind::ObservationRead(Box::new(cause)),
            };
            return Some(WorthQueryOutputDemandDenial::new(kind, subject));
        }
        self.refused.borrow_mut().take().map(|stop| {
            crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(stop).demand_stop()
        })
    }

}
