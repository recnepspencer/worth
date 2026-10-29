//! Gate 8.7 — safe-retry re-dispatch through the real bank-external-rail (§11 rows 16–18).
//!
//! Authority precedes every transport call. Exactly-once is observed through
//! the rail's admission ledger and independent physical consequence owner,
//! not by Query declining to ask (R8.70).

pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::LedgerStatus;
pub use bank_server::{
    BankEstateProgressionDenial, BankRecoveryClaimStatus, BankRecoveryDenialKind,
    BankRecoveryDurability,
};
pub use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectPostureKind;

pub use super::phase8_cross_gate::world;
pub use crate::facade::server::authorization_time::AuthorizationTimeController;
pub use crate::facade::server::support::request_scope;

pub fn assert_request_affine_retry_completed_once(
    world: &world::CrossGateWorld,
    correlation: &bank_external_rail::RailCorrelation,
) {
    let dispatches = world.transport.production_dispatches();
    assert_eq!(
        dispatches.len(),
        2,
        "initial and retry must both cross the production Bank adapter"
    );
    assert_eq!(dispatches[0].correlation, dispatches[1].correlation);
    assert_eq!(dispatches[0].payload, dispatches[1].payload);
    assert_eq!(&dispatches[0].correlation, correlation);
    assert_eq!(
        world.transport.completed_effect_count(),
        1,
        "two request-affine dispatches must produce one physical consequence"
    );
    let completed = world
        .transport
        .completed_notice(correlation)
        .expect("the independent consequence owner retains the completed notice");
    assert_eq!(
        (completed.estate(), completed.notice(), completed.subject()),
        (
            world.fixture.estate.get(),
            world.fixture.notice.get(),
            world.fixture.deceased.get(),
        )
    );
}
