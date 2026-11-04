//! A copied Native root does not transfer the original product's retry affinity.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial as Denial;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind;

#[test]
fn inherited_directory_key_refuses_a_sibling_product_affinity() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = pending_support::open(execution, contacts.clone());
    let (_, live) = scope();
    let original = set(home.runtime(), &live, 0x7375626c696e67, 8).unwrap();
    assert!(matches!(original, Outcome::Committed { result: 8, .. }));
    let application = home.runtime().runtime();
    let sibling = application
        .with_application_advancement(&live, |phase| {
            application
                .branches()
                .fork(application.current_world())
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
        })
        .unwrap()
        .expect("the actual directory World creates the exact sibling");
    let principal =
        identity::inspection_authentication::authenticate(application.installed_schema(), &live);
    let retry = home
        .runtime()
        .request(&principal, &live)
        .on_branch(sibling)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".to_owned(),
            balance: 8,
        })
        .without_source()
        .idempotency(&0x7375626c696e67_u64)
        .execute_in_program(
            home.runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    match retry {
        Err(Denial::Idempotency(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
        ),
        other => panic!("the inherited key belongs to its original product: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    drop(original);
    drop(home);
}

#[test]
fn reopened_home_refuses_the_prior_open_branch_selector_before_effect() {
    use worth_query_host::facade::primary_graph::{
        WorthQueryProductBranchAdmissionDenial as BranchDenial,
        WorthQueryProductBranchAdmissionRefusalKind,
    };
    let contacts = Arc::new(AtomicUsize::new(0));
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        location.clone(),
        build,
        Arc::new(AtomicUsize::new(0)),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|refused| opening_failed("stale-selector writer Home", &refused)),
        )
    })
    .unwrap();
    let stale = home.runtime().runtime().current_world();
    let (_, live) = scope();
    assert!(matches!(
        set(home.runtime(), &live, 0x7374616c65, 8).unwrap(),
        Outcome::Committed { .. }
    ));
    // Ordinary abandonment makes no D11 close claim. The opaque supplier's
    // retirement belongs to its owner; only the old selector survives here.
    drop(home);
    let request =
        fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
            panic!("existing Home cannot initialize")
        });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|refused| opening_failed("stale-selector reader Home", &refused)),
        )
    })
    .unwrap();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &live,
    );
    let answer = home
        .runtime()
        .request(&principal, &live)
        .on_branch(stale)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".to_owned(),
            balance: 8,
        })
        .without_source()
        .idempotency(&0x7374616c65_u64)
        .execute_in_program(
            home.runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    match answer {
        Err(Denial::ProductSelection(denial)) => assert_eq!(
            denial,
            BranchDenial::Refused(WorthQueryProductBranchAdmissionRefusalKind::StaleIncarnation)
        ),
        other => panic!("the previous open's selector grants no current admission: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    assert!(
        matches!(
            set(home.runtime(), &live, 0x7374616c65, 8).unwrap(),
            Outcome::RecoveredCommitted(_)
        ),
        "fresh admission still resolves the original durable occurrence"
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
}

mod capability_home {
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
        path: std::path::PathBuf,
        initialize: bool,
    ) -> WorthQueryApplicationHomeRequest<'static, HostIdentitySchema, CapabilityProgram> {
        let declaration = HostIdentitySchema::declaration().unwrap();
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(path),
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
            request(path.clone(), true).open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| opening_failed("capability writer", &refused));
        let app = home.runtime().runtime();
        let external =
            identity::inspection_authentication::authenticate(app.installed_schema(), &live);
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
            request(path, false).open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| opening_failed("capability reader", &refused));
        let app = home.runtime().runtime();
        let external =
            identity::inspection_authentication::authenticate(app.installed_schema(), &live);
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
}
