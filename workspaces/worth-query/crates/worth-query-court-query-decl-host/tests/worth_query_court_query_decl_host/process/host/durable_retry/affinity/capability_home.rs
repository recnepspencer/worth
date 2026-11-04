use super::*;
use identity::HostIdentitySchema;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_installation::{
        WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationLimits,
    },
    primary_graph::{SignalConditionalEvaluationBudget, WorthQueryPrincipalResolutionMode},
    runtime::WorthQueryApplicationQueryResourceProfile,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
struct CapabilityProgram;
struct InspectionFeature;
impl ApplicationFeature<HostIdentitySchema> for InspectionFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.host.incarnation.inspection-feature.v1";
}
impl ApplicationProgramDefinition<HostIdentitySchema> for CapabilityProgram {
    type Contributions = (identity::HostIdentitySurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.host.incarnation.inspection.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<HostIdentitySchema, InspectionFeature>().finish()]
    }
}
fn request(
    location: WorthQueryApplicationHomeLocation,
    initialize: bool,
) -> WorthQueryApplicationHomeRequest<'static, HostIdentitySchema, CapabilityProgram> {
    let declaration = HostIdentitySchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        location,
        ApplicationProgramAuthoring::<HostIdentitySchema, CapabilityProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        WorthQueryInMemoryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        move |_, graph, schema| {
            assert!(initialize, "existing capability Home cannot initialize");
            identity::seed_identity_graph(graph, schema)
        },
    )
}
#[test]
fn reopened_home_refuses_prior_open_capability_before_disclosing_new_observations() {
    use identity::{
        comparison_model::{GovernedAccountRead, GovernedAccountReadBinding},
        inspection_capability::{InspectAccountCapability, InspectAccountOperation},
    };
    use worth_query_host::facade::causal_inspection::WorthQueryApplicationInspectionStopKind;
    let path = fixture::location();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let cancellation = WorthQueryCancellationSource::new();
    let live = WorthQueryRequestScope::new(None, cancellation.token());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(
            WorthQueryApplicationHomeLocation::Directory(path.clone()),
            true,
        )
        .open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("capability writer", &refused));
    let app = home.runtime().runtime();
    let external = identity::inspection_authentication::authenticate(app.installed_schema(), &live);
    let access = app
        .with_application_advancement(&live, |phase| {
            let selected = app.on_branch(app.current_world()).select(&phase).unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &app.installed_schema()
                        .principal_binding(identity::IdentityBinding::reference())
                        .unwrap(),
                    &external,
                    &live,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &app.installed_schema()
                        .capability(
                            InspectAccountCapability::reference(),
                            InspectAccountOperation::reference(),
                        )
                        .unwrap(),
                    "account-001".to_owned(),
                    &live,
                )
                .unwrap()
        })
        .unwrap();
    drop(external);
    // The actual admitted capability survives; no proof or owner is fabricated.
    drop(home);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(WorthQueryApplicationHomeLocation::Directory(path), false)
            .open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("capability reader", &refused));
    let app = home.runtime().runtime();
    let external = identity::inspection_authentication::authenticate(app.installed_schema(), &live);
    let request = home.runtime().request(&external, &live);
    let read = || {
        request
            .query(GovernedAccountRead("account-001".to_owned()))
            .retain_projection_for_inspection()
            .execute_governed(
                InspectAccountCapability::reference(),
                InspectAccountOperation::reference(),
                "account-001".to_owned(),
            )
            .unwrap()
    };
    let before = read();
    let after = read();
    assert_eq!(before.rows(), &["account-001".to_owned()]);
    assert_eq!(after.rows(), before.rows());
    let result = request
        .inspect::<GovernedAccountReadBinding>(after.receipt().inspect().observation().unwrap())
        .why_changed(before.receipt().inspect().observation().unwrap())
        .reference_only()
        .execute_with_diagnostic(access);
    let denial = match result {
        Err(denial) => denial,
        Ok(_) => panic!("prior Home capability cannot disclose new observations"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::StaleIncarnation
    );
    let current = read();
    assert_eq!(current.rows(), before.rows());
}

mod trusted_time;
