use super::*;
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
    time::{Duration, SystemTime},
};
use worth_query_host::facade::declaration::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::{admission::authenticated_principal as authentication, runtime::*};

pub fn configuration() -> program::ExampleConfiguration {
    program::ExampleConfiguration {
        source: PlanarSourceRuntime::new(PlanarSourceAdjustmentHandler),
        topology: TopologyConfiguration {
            final_feature: planar_final_feature_runtime(),
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
        },
    }
}

#[expect(
    clippy::result_large_err,
    reason = "Keep complete typed Query refusal evidence, following the Query workspace policy"
)]
#[allow(
    dead_code,
    reason = "The default-policy example and shared-service example compile this module independently"
)]
pub fn install<const ERASE: bool>(
    configuration: program::ExampleConfiguration,
    seeded: impl FnOnce(),
) -> Result<
    installation::WorthQueryProgramApplicationRuntime<ExampleSchema, ExampleProgram<ERASE>>,
    installation::WorthQueryInMemoryApplicationDenial,
> {
    install_configured::<ERASE>(
        configuration,
        installation::ApplicationBuildIdentity::parse("27120000-0000-4000-8000-000000000001")
            .unwrap(),
        limits(),
        seeded,
    )
}

#[expect(
    clippy::result_large_err,
    reason = "Keep complete typed Query refusal evidence, following the Query workspace policy"
)]
pub fn install_configured<const ERASE: bool>(
    configuration: program::ExampleConfiguration,
    build_identity: installation::ApplicationBuildIdentity,
    limits: installation::WorthQueryInMemoryApplicationLimits,
    seeded: impl FnOnce(),
) -> Result<
    installation::WorthQueryProgramApplicationRuntime<ExampleSchema, ExampleProgram<ERASE>>,
    installation::WorthQueryInMemoryApplicationDenial,
> {
    let program = ApplicationProgramAuthoring::<ExampleSchema, ExampleProgram<ERASE>>::begin()
        .validated_program()
        .unwrap();
    installation::in_memory_program(
        program,
        ExampleSchema::declaration().unwrap(),
        (configuration,),
        build_identity,
        limits,
        |_, graph, installed| {
            seeded();
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<ExampleSchema>())
                .unwrap();
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for (name, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
                let key = format!("anchor-{name}");
                graph.bind_entity(
                    primary_graph::WorthQueryApplicationEntitySeed::new(
                        Body::reference::<ExampleSchema>(),
                        entity_key(&key),
                    )
                    .field(BodyKey::reference(), key)
                    .field(Length::reference(), length(y + 1))
                    .field(PositionX::reference(), length(x))
                    .field(PositionY::reference(), length(y)),
                )?;
            }
            for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
                graph.bind_relation(primary_graph::WorthQueryApplicationRelationSeed::new(
                    PlanarSuccessor::reference::<ExampleSchema>(),
                    format!("successor-{from}-{to}"),
                    entity_key(&format!("anchor-{from}")),
                    entity_key(&format!("anchor-{to}")),
                ))?;
            }
            Ok(())
        },
    )
}
fn entity_key(key: &str) -> primary_graph::WorthQueryApplicationEntityKey<ExampleSchema, Body> {
    primary_graph::WorthQueryApplicationEntityKey::new(key).unwrap()
}
pub fn length(value: u64) -> worth_query_consumer_values::PositiveLength {
    worth_query_consumer_values::PositiveLength::new(value).unwrap()
}
#[allow(
    dead_code,
    reason = "The default-policy example and shared-service example compile this module independently"
)]
fn limits() -> installation::WorthQueryInMemoryApplicationLimits {
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    );
    limits_for_policy(policy)
}

pub fn limits_for_policy(
    policy: ExecutionRequestPolicy,
) -> installation::WorthQueryInMemoryApplicationLimits {
    let world = WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 4,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 32,
                history_metadata_bytes: 524_288,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 32,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 4,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 4,
                retained_partial_metadata_bytes: 524_288,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 64,
                in_flight_pin_acquisition_reservations: 16,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 16,
            },
        },
        WorthQueryProductWorldClock::start(),
        WorthQueryInvalidationResources::new(),
        policy,
    )
    .unwrap();
    installation::WorthQueryInMemoryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(32).unwrap()),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(
        "https://required-feature.invalid/local",
        "model-owner",
    )
    .unwrap()
}
struct LocalIdentity;
impl authentication::WorthQueryAuthenticationAdapter for LocalIdentity {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "required-feature-local-authentication"
    }
    fn validate<'a>(
        &'a self,
        _: (),
        _: &'a authentication::WorthQueryRequestScope,
    ) -> authentication::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async {
            let now = SystemTime::now();
            authentication::WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                authentication::WorthQueryAuthenticationAudience::new("required-feature").unwrap(),
                authentication::WorthQueryAuthenticationMethod::new("local").unwrap(),
                now,
                now + Duration::from_secs(600),
                Vec::new(),
            )
            .map_err(|_| {
                authentication::WorthQueryAuthenticationAdapterFailure::new(
                    authentication::WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}
pub fn authenticate(
    application: &Application,
) -> (
    authentication::WorthQueryRequestScope,
    authentication::WorthQueryAuthenticatedExternalPrincipal<ExampleSchema>,
) {
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
    let adapter = authentication::admit_authentication_adapter(
        application.installed_schema(),
        authentication::WorthQueryAuthenticationAdapterAdmission::new(
            authentication::WorthQueryAuthenticationAudience::new("required-feature").unwrap(),
            authentication::WorthQueryAuthenticationMethod::new("local").unwrap(),
        ),
        LocalIdentity,
    )
    .unwrap();
    let principal = {
        let mut future = pin!(adapter.authenticate((), &scope));
        match future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(result) => result.unwrap(),
            Poll::Pending => panic!("local authentication completes immediately"),
        }
    };
    (scope, principal)
}
