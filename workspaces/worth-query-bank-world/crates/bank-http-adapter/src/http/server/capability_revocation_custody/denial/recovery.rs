//! Recovery describes the still-retained command; denial never discharges its owner.
use super::*;

pub(super) fn managed(cause: WorthQueryManagedApplicationRecoveryDenial) -> BankHttpDenial {
    use WorthQueryManagedApplicationRecoveryDenial as D;
    match cause {
        D::ExecutionDenied(cause) => advancement_denial::advancement(cause),
        D::Authorization(cause) => authorization_denial::authorization_denial(cause.kind().into()),
        D::ProviderAdmission(cause) => crate::http::server::world_denial::native_admission(cause),
        D::Demand(cause) => query(WorthQueryApplicationRequestQueryDenial::OutputSettlement(
            cause,
        )),
        D::World(cause) => crate::http::server::world_denial::recovery(cause),
        D::Adoption(cause) => match *cause {
            RuntimeWorldSettledRelationalAdoptionDenial::Recovery(cause) => {
                crate::http::server::world_denial::recovery(cause)
            }
            RuntimeWorldSettledRelationalAdoptionDenial::Publication(cause) => {
                crate::http::server::world_denial::publication(cause.cause())
            }
        },
        D::ProviderBusy | D::ProviderCapacity | D::ProductUnavailable => unavailable(),
        D::ForeignAdmission | D::BindingMismatch => operator(),
    }
}
