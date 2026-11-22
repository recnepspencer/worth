//! A governed where filter consults its field for membership: admission
//! requires the same internal influence as `where_equal`.
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::contract_admission::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::{
    FilterMembershipQuery, FilterRowPresenceOnlyQuery,
};

macro_rules! admit_filtered {
    ($query:ty) => {{
        let context = admission_context();
        let query = context
            .world
            .application
            .installed_schema()
            .certification_query(<$query>::reference())
            .unwrap();
        context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                let capability = admit_touch_account_capability(
                    &(host_custody.phase_for_application_for_test(&context.world.application))
                        .test_host_request_for_test(),
                    &context.world,
                    &context.principal,
                    &context.request,
                )
                .unwrap();
                let access = WorthQueryApplicationQueryAccessContext::new(
                    &context.principal,
                    &context.account,
                );
                context
                    .world
                    .selected_product(
                        &(host_custody.phase_for_application_for_test(&context.world.application))
                            .test_host_request_for_test(),
                    )
                    .admit_governed_application_query(
                        &host_custody.phase_for_application_for_test(&context.world.application),
                        &query,
                        &access,
                        capability,
                        ApplicationQueryParameterSet::new()
                            .bind(status_parameter(), "open".to_owned())
                            .expect("fixture query parameter must encode"),
                        controls(&context.request),
                    )
                    .map(|_| ())
                    .map_err(|denial| denial.kind())
            })
            .expect("the fixture block owns its installed host call")
    }};
}

#[test]
fn a_filter_field_permitting_only_row_presence_refuses_governed_admission() {
    assert_eq!(
        admit_filtered!(FilterRowPresenceOnlyQuery),
        Err(WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid)
    );
}

#[test]
fn a_filter_field_permitting_every_membership_surface_is_admitted() {
    assert_eq!(admit_filtered!(FilterMembershipQuery), Ok(()));
}
