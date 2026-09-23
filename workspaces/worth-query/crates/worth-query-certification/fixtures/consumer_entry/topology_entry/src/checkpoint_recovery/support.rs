use std::future::Future;
use std::pin::pin;
use std::sync::{atomic::AtomicUsize, Arc};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::{
    declaration::authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
    },
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
    },
    runtime::{
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
        RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
        RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
        RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
        WorthQueryApplicationCandidateResourceProfile, WorthQueryApplicationQueryResourceProfile,
        WorthQueryProductWorldClock, WorthQueryProductWorldResources,
    },
};

use super::*;

pub(super) type Application = application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    CheckpointProgram,
>;

pub(super) fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
) -> Application {
    let configuration = (TopologyConfiguration {
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
    },);
    let program = ApplicationProgramAuthoring::<CheckpointSchema, CheckpointProgram>::begin()
        .validated_program()
        .expect("the checkpoint program is complete");
    let declaration = CheckpointSchema::declaration().expect("the checkpoint schema is valid");
    match checkpoint {
        Some(checkpoint) => application_installation::in_memory_program_from_checkpoint(
            program,
            declaration,
            configuration,
            limits(),
            checkpoint,
        )
        .expect("the checkpoint restores"),
        None => application_installation::in_memory_program(
            program,
            declaration,
            configuration,
            limits(),
            |graph, installed| {
                let principal = installed
                    .principal_binding(ConsumerPrincipalBinding::reference::<CheckpointSchema>())
                    .expect("the principal mapping is installed");
                graph.bind_principal(
                    &principal,
                    primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                    1_u64,
                    external_identity(),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
                seed_cycle(graph);
                Ok(())
            },
        )
        .expect("the checkpoint source installs"),
    }
}

fn seed_cycle(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>) {
    for (name, x, y) in [
        ("a", 1, 1),
        ("b", 10, 1),
        ("c", 1, 10),
        ("isolated", 50, 50),
        ("island", 60, 50),
        ("atoll", 50, 60),
    ] {
        let key = format!("anchor-{name}");
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Body::reference::<CheckpointSchema>(),
                    entity_key(&key),
                )
                .field(BodyKey::reference::<CheckpointSchema>(), key)
                .field(Length::reference::<CheckpointSchema>(), length(1))
                .field(PositionX::reference::<CheckpointSchema>(), length(x))
                .field(PositionY::reference::<CheckpointSchema>(), length(y)),
            )
            .unwrap();
    }
    for (from, to) in [
        ("a", "b"),
        ("b", "c"),
        ("c", "a"),
        ("isolated", "island"),
        ("island", "atoll"),
        ("atoll", "isolated"),
    ] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<CheckpointSchema>(),
                format!("anchor-{from}-to-{to}"),
                entity_key(&format!("anchor-{from}")),
                entity_key(&format!("anchor-{to}")),
            ))
            .unwrap();
    }
}

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}

pub(super) fn length(value: u64) -> PositiveLength {
    PositiveLength::new(value).unwrap()
}

fn limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        WorthQueryProductWorldResources::install(
            RuntimeWorldBudgetInstallation {
                branches: RuntimeWorldBranchBudgetInstallation {
                    live_product_branches: 4,
                },
                history: RuntimeWorldHistoryBudgetInstallation {
                    retained_composite_commits: 32,
                    history_metadata_bytes: 524_288,
                },
                observations: RuntimeWorldObservationBudgetInstallation {
                    active_observations: 16,
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
        )
        .unwrap(),
        WorthQueryApplicationCandidateResourceProfile::bounded(4_096, 8_192, 4_096).unwrap(),
        WorthQueryApplicationQueryResourceProfile::bounded(4_096, 4_096, 4_096, 32).unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

struct LocalIdentityAdapter;
struct LocalCredential;

impl authentication::WorthQueryAuthenticationAdapter for LocalIdentityAdapter {
    type Credential = LocalCredential;

    fn configuration_identity(&self) -> &str {
        "worth.query.certification.checkpoint-authentication.v1"
    }

    fn validate<'a>(
        &'a self,
        _: Self::Credential,
        _: &'a authentication::WorthQueryRequestScope,
    ) -> authentication::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            authentication::WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                authentication::WorthQueryAuthenticationAudience::new("checkpoint").unwrap(),
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

fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://checkpoint.invalid/local", "model-owner")
        .unwrap()
}

pub(super) fn authenticate(
    application: &Application,
) -> (
    authentication::WorthQueryRequestScope,
    authentication::WorthQueryAuthenticatedExternalPrincipal<CheckpointSchema>,
) {
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let scope = authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    );
    let adapter = authentication::admit_authentication_adapter(
        application.installed_schema(),
        authentication::WorthQueryAuthenticationAdapterAdmission::new(
            authentication::WorthQueryAuthenticationAudience::new("checkpoint").unwrap(),
            authentication::WorthQueryAuthenticationMethod::new("local").unwrap(),
        ),
        LocalIdentityAdapter,
    )
    .unwrap();
    let principal = block_on(adapter.authenticate(LocalCredential, &scope)).unwrap();
    (scope, principal)
}

fn block_on<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("the local authentication completes synchronously"),
    }
}
