//! Finite notification capability in the otherwise unchanged real Bank seed.
use super::*;
use bank_domain::estate::{CapabilityGrantId, CapabilityValidity, EstateMoment};

pub(in crate::http::server::tests) fn application_with_expiring_notification(
    account: AccountId,
    source: impl WorthQueryRuntimeTimeSource,
    valid_until: u64,
) -> CausalHttpApplication {
    let mut estate = estate_world(
        InstitutionId::new(1).unwrap(),
        BankPrincipalId::new(1).unwrap(),
    );
    let mut grant = *estate.grant(CapabilityGrantId::new(14).unwrap()).unwrap();
    grant.scope.validity = CapabilityValidity::new(
        EstateMoment::from_epoch_seconds(0),
        EstateMoment::from_epoch_seconds(valid_until),
    )
    .unwrap();
    estate = estate.with_grant(grant);
    application_from_timed_seed(world_seed(account).estate(estate), source)
}
