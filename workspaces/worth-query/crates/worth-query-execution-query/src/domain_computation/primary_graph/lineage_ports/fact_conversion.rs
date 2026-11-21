//! Phase-bound constructor on `SourceFactExpansionControl`.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;

use crate::domain_computation::primary_graph::application_query::observed_source::fact_conversion::SourceFactExpansionControl;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind,
};

pub trait SourceFactExpansionControlExt<'request, 'authority>: Sized {
    fn for_phase<S: worth_query_installation::facade::ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &'request WorthQueryAdvancementPhase<'authority>,
        request: Option<&'request WorthQueryRequestScope>,
        subject: &str,
    ) -> Result<Self, WorthQuerySourceExpectationDenial>;
}

impl<'request, 'authority> SourceFactExpansionControlExt<'request, 'authority>
    for SourceFactExpansionControl<'request, 'authority>
{
    fn for_phase<S: worth_query_installation::facade::ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &'request WorthQueryAdvancementPhase<'authority>,
        request: Option<&'request WorthQueryRequestScope>,
        subject: &str,
    ) -> Result<Self, WorthQuerySourceExpectationDenial> {
        let execution = phase
            .execution_request_for(&runtime.runtime.product_runtime)
            .map_err(|_| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::ForeignOperation,
                    subject,
                )
            })?;
        Ok(Self::in_request(execution, request))
    }
}
