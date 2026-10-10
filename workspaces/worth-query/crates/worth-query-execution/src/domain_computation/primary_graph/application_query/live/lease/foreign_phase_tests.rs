use crate::domain_computation::primary_graph::tests::fixture::{
    admit_touch_account_capability, governed_live_account_parameters,
    installed_authorization_world, installed_capability_live_world, live_account_parameters,
    live_scope, Account, AccountIdentity, AccountSummaryParameters, Activity,
    GovernedLiveAccountActivityCause, GovernedLiveAccountActivityQuery,
    GovernedLiveAccountActivityResult, LiveAccountActivityCause, LiveAccountActivityQuery,
    LiveAccountActivityResult,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationLiveControls, WorthQueryApplicationLiveOpenDenialKind,
    WorthQueryPrincipalResolutionMode,
};
use std::time::{Duration, Instant, UNIX_EPOCH};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn foreign_phase_refuses_both_live_doors_before_cancelled_read_admission() {
    for governed in [false, true] {
        let issuer = installed_authorization_world(true);
        let receiver = if governed {
            installed_capability_live_world()
        } else {
            installed_authorization_world(true)
        };
        if governed {
            receiver.authorization_time.script([
                UNIX_EPOCH + Duration::from_secs(100),
                UNIX_EPOCH + Duration::from_secs(100),
            ]);
        }
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(60),
            cancellation.token(),
        );
        let external = receiver.authenticate("alice", Duration::from_secs(60), &request);
        let principal = receiver
            .selected_product()
            .resolve_authenticated_principal(
                &receiver.binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let account = receiver
            .selected_product()
            .resolve_entity(
                AccountIdentity::reference(),
                "account-1".to_owned(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let capability = governed
            .then(|| admit_touch_account_capability(&receiver, &principal, &request).unwrap());
        let before = receiver.application.provider_session_resource_count();
        cancellation.cancel();
        issuer.application.with_application_advancement(&live_scope(), |phase| {
            let controls = WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048).unwrap();
            let denial = if governed {
                let query = receiver.application.installed_schema().certification_query(GovernedLiveAccountActivityQuery::reference()).unwrap();
                receiver.selected_product().open_governed_application_query_live::<
                    GovernedLiveAccountActivityQuery, AccountSummaryParameters, GovernedLiveAccountActivityResult,
                    _, _, Account, Activity, GovernedLiveAccountActivityCause, _, _, _,
                >(&phase, query, &principal, account, capability.unwrap(), governed_live_account_parameters("account-1"), controls)
                .err().expect("a foreign phase cannot open a governed live lease")
            } else {
                let query = receiver.application.installed_schema().certification_query(LiveAccountActivityQuery::reference()).unwrap();
                receiver.selected_product().open_application_query_live::<
                    LiveAccountActivityQuery, AccountSummaryParameters, LiveAccountActivityResult,
                    _, _, Account, Activity, LiveAccountActivityCause,
                >(&phase, query, &principal, account, live_account_parameters("account-1"), controls)
                .err().expect("a foreign phase cannot open an ordinary live lease")
            };
            assert_eq!(denial.kind(), WorthQueryApplicationLiveOpenDenialKind::ForeignAdvancementPhase);
        }).unwrap();
        assert_eq!(
            receiver.application.provider_session_resource_count(),
            before
        );
    }
}
