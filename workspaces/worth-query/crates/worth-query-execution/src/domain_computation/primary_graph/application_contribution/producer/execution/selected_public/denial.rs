//! Preserve the current-control or original D origin of a producer admission stop.
use super::*;
pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn preparation_stop<
    Schema,
    Binding,
>(
    stop: SelectedPublicPreparationStop,
) -> ProducerExecutionStop
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    use crate::domain_computation::authorization::SelectedOperationGraphWorkStop as GraphStop;

    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;

    let resource = |stop| source_readmission::ready_resource_denial("", stop);
    let work = || denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "").into();
    let request =
        |kind| super::super::denial::request_admission_rejected(denial(kind, Binding::IDENTITY));
    let interrupted = |kind| match kind {
        WorthQueryRequestInterruption::Cancelled => {
            request(WorthQueryOutputDemandDenialKind::Cancelled)
        }
        WorthQueryRequestInterruption::DeadlineExceeded => {
            request(WorthQueryOutputDemandDenialKind::TimedOut)
        }
    };
    let current = |stop: ProducerExecutionStop| {
        super::super::denial::request_admission_rejected(
        stop.into_output_denial().with_recovery_posture(
            crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture::Retryable))
    };
    match stop {
        SelectedPublicPreparationStop::CurrentPrincipal(cause) => {
            current(preparation_stop::<Schema, Binding>(
                SelectedPublicPreparationStop::Principal(cause),
            ))
        }
        SelectedPublicPreparationStop::CurrentScope(cause) => {
            current(preparation_stop::<Schema, Binding>(
                SelectedPublicPreparationStop::Scope(cause),
            ))
        }
        SelectedPublicPreparationStop::CurrentHead(cause) => current(cause),
        SelectedPublicPreparationStop::CurrentMovedHead => {
            current(preparation_stop::<Schema, Binding>(
                SelectedPublicPreparationStop::MovedHead,
            ))
        }
        SelectedPublicPreparationStop::CurrentOperation(cause) => {
            // Authorization uses C, but graph-work setup owns D. Only current
            // admission and request interruptions may acquire retry posture here.
            let retryable = matches!(
                &cause,
                SelectedConventionalAdmissionStop::Authorization(_)
                    | SelectedConventionalAdmissionStop::Interrupted(_)
                    | SelectedConventionalAdmissionStop::ExpiredAuthentication
                    | SelectedConventionalAdmissionStop::AccountingOverflow
                    | SelectedConventionalAdmissionStop::GraphWork(GraphStop::Interrupted(_))
            );
            let stop = preparation_stop::<Schema, Binding>(
                SelectedPublicPreparationStop::Operation(cause),
            );
            if retryable {
                current(stop)
            } else {
                stop
            }
        }
        SelectedPublicPreparationStop::ForeignSelectedRuntime => denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            Binding::IDENTITY,
        )
        .into(),

        SelectedPublicPreparationStop::Issuer => denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into(),
        SelectedPublicPreparationStop::Principal(denial) => {
            super::super::denial::principal_rejected(Binding::IDENTITY, denial)
        }
        SelectedPublicPreparationStop::Scope(denial) => {
            super::super::denial::scope_rejected(Binding::IDENTITY, denial)
        }
        SelectedPublicPreparationStop::UnsupportedScopeMode => denial(
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
            Binding::IDENTITY,
        )
        .into(),
        SelectedPublicPreparationStop::Head(stop) => stop,
        SelectedPublicPreparationStop::MovedHead => denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into(),
        SelectedPublicPreparationStop::Operation(stop) => match stop {
            SelectedConventionalAdmissionStop::AccountingOverflow => work(),
            SelectedConventionalAdmissionStop::Authorization(denial) => {
                super::super::denial::request_authority_denied(Binding::IDENTITY, denial)
            }
            SelectedConventionalAdmissionStop::Interrupted(kind) => interrupted(kind),
            SelectedConventionalAdmissionStop::ExpiredAuthentication => {
                request(WorthQueryOutputDemandDenialKind::ProducerUnavailable)
            }

            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Source(stop)) => resource(stop),
            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Interrupted(kind)) => {
                interrupted(kind)
            }

            _ => request(WorthQueryOutputDemandDenialKind::ProducerUnavailable),
        },
    }
}
