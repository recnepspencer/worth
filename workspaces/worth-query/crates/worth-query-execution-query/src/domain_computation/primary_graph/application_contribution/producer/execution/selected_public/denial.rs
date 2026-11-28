//! Map a selected producer's admission stop to its execution denial.
use super::*;
pub fn preparation_stop<Schema, Binding>(
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
    match stop {
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
