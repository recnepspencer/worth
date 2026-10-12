use super::*;

#[test]
fn result_disclosure_rule_cannot_open_a_predicate_read() {
    let context = admission_context();
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(ResultRulePredicateQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let capability = admit_touch_account_capability(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let denial = context
                .world
                .selected_product(
                    &(host_custody.phase_for_application(&context.world.application))
                        .test_host_request(),
                )
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .expect("fixture query parameter must encode"),
                    controls(&context.request),
                )
                .err()
                .expect("result disclosure must not counterfeit internal authority");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            );
        })
        .expect("the fixture block owns its installed host call")
}
