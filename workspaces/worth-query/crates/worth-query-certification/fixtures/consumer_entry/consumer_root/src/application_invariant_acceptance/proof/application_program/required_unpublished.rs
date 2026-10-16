//! The ordinary fixed journey owns every native handoff and refused close.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_decl::facade::application_program::{
    ApplicationActionReference, ApplicationCompositionInstance,
};
use worth_query_host::facade::application_entry::WorthQueryOperationConstructionDenial;
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

struct MissingActionOccurrence;
impl ApplicationCompositionInstance for MissingActionOccurrence {
    const PATH: &'static str = "uninstalled-source-occurrence";
}
type MissingFixedAction = ApplicationActionReference<
    ConsumerSchema,
    MissingActionOccurrence,
    worth_query_topology_entry::PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    crate::application_program::ConsumerFixedDelivery,
>;

use worth_query_decl::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationCloseDenial,
    WorthQueryOperationDeliveryDenial, WorthQueryOperationProgress as Progress,
};
use worth_query_host::facade::primary_graph::RuntimeWorldRecoveryDenial;
use worth_query_host::facade::runtime::ProductUnpublishedCause;
use worth_query_topology_entry::{
    PlanarOutputRead, PlanarSourceAdjustmentBinding, planar_source_adjustment_contacts,
    reset_planar_source_adjustment_contacts,
};

// This consumer cannot clone its initiating intent.
struct OriginalAdjustment(PlanarSourceAdjustment);
impl ApplicationMutationIntent<ConsumerSchema> for OriginalAdjustment {
    type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;
    fn input(&self) -> &PlanarSourceAdjustment {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<ConsumerSchema>>::ScopeBinding {
        <PlanarSourceAdjustment as ApplicationMutationIntent<ConsumerSchema>>::scope_binding(
            &self.0,
        )
    }
}

pub(in crate::application_invariant_acceptance::proof::application_program) fn original_partial_recovers_required_outputs(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    use crate::application_program::fixed_successors::{RemovedFixed, UnchangedFixed};
    use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
    let roster = worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
        .support(ApplicationProgramAuthoring::<ConsumerSchema, UnchangedFixed>::begin().validated_program().unwrap())
        .support(ApplicationProgramAuthoring::<ConsumerSchema, RemovedFixed>::begin().validated_program().unwrap());
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<crate::ConsumerProgram>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<UnchangedFixed>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<RemovedFixed>(
            "model-owner",
            "model-owner",
        ),
    ];
    let mut world =
        installation::install_with_roster(foreign, 8192, 4096, roster, &grants).unwrap();
    let request_runtime = world.application.clone();
    let constructing = std::mem::replace(&mut world.application, request_runtime);
    let adapter = authentication::admit(world.application.installed_schema());
    let contact_identity = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&contact_identity);
    let operation = {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        let description = request
            .mutate(OriginalAdjustment(contact_identity.clone()))
            .expect_source(source)
            .idempotency_owned(10_062)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"));
        let (description, denial) = description
            .retain_operation::<crate::ConsumerProgram, MissingFixedAction>(&constructing)
            .err()
            .expect("an uninstalled exact occurrence refuses construction");
        assert!(matches!(
            denial,
            WorthQueryOperationConstructionDenial::Action(
                worth_query_host::facade::domain::WorthQueryActionResolutionDenial::Missing
            )
        ));
        assert_eq!(planar_source_adjustment_contacts(&contact_identity), (0, 0));
        description
            .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&constructing)
            .unwrap_or_else(|(_, denial)| {
                panic!("returned original description constructs: {denial:?}")
            })
    };
    // Initiating authority is gone before the first native admission.
    world
        .application
        .with_host_advancement(|phase| {
            world.application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap();
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let operation = match operation.advance(&constructing.request(&principal, &scope)) {
        Advance::Progress {
            operation,
            progress: Progress::SourceUnpublished,
        } => operation,
        _ => panic!("the actual append fault retains original fixed partial custody"),
    };
    drop(constructing);
    assert_eq!(planar_source_adjustment_contacts(&contact_identity), (1, 1));
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );

    let mut held_partial = None;
    let mut operation = Some(operation);
    let settled = (0..32)
        .find_map(|_| {
            let fresh_scope = authentication::request_scope();
            let fresh_principal = authentication::block_on(adapter.authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &fresh_scope,
            ))
            .unwrap();
            let current = operation
                .take()
                .expect("each progress returns its complete operation");
            match current.advance(&world.application.request(&fresh_principal, &fresh_scope)) {
                Advance::Progress { operation: next, progress } => {
                    if matches!(progress, Progress::Recovery(worth_query_host::facade::application_entry::WorthQueryProgramSourceRecoveryProgress::Performed)) {
                        // A post-recovery view blocks record release, rather than recovery.
                        let catalog = world.application.product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap()).unwrap();
                        assert_eq!(catalog.rows().len(), 1);
                        let inspector = world.application.readmit_product_publication_recovery(catalog.rows()[0].handle()).unwrap();
                        let partial = inspector.inspect().unwrap();
                        assert_eq!(partial.cause(), ProductUnpublishedCause::SettlementPending);
                        held_partial = Some(partial);
                    }
                    let next = if matches!(progress, Progress::SourcePromoted) {
                        let request = world.application.request(&fresh_principal, &fresh_scope);
                        adopt(&request, *world.application.supported_program::<RemovedFixed>().unwrap().owned_revision());
                        let next = match next.advance(&request) {
                            Advance::Refused { operation, denial: worth_query_host::facade::application_entry::WorthQueryOperationAdvanceDenial::Action(_) } => operation,
                            _ => panic!("changed installed action refuses before output progression"),
                        };
                        adopt(&request, *world.application.supported_program::<UnchangedFixed>().unwrap().owned_revision());
                        next
                    } else { next };
                    operation = Some(next);
                    None
                }
                Advance::Settled(settled) => Some(settled),
                Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                    panic!("original fixed native progression: {denial:?}")
                }
            }
        })
        .expect("the native graph settles within its authored bound");
    assert!(
        held_partial.is_some(),
        "the real caller capability remains live for close refusal"
    );
    assert!(
        settled.handler_result().is_none(),
        "recovery never invents a handler reply"
    );
    assert!(settled.recovered_progress().is_some());
    assert_eq!(settled.settlement().output_count(), 4);
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        1
    );
    assert_eq!(planar_source_adjustment_contacts(&contact_identity), (1, 1));
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );

    // Independent retained observations exhaust actual World read capacity.
    let request = world.application.request(&principal, &scope);
    let mut observations = Vec::new();
    for _ in 0..32 {
        match request.retain_current_read() {
            Ok(observation) => observations.push(observation),
            Err(_) => break,
        }
    }
    assert!(!observations.is_empty());
    assert!(matches!(
        settled.read_current(&request),
        Err(WorthQueryOperationDeliveryDenial::Observation(_))
    ));
    drop(observations);
    use worth_query_host::facade::application_entry::WorthQueryOperationDeliveryPolicy;
    let one = NonZeroUsize::new(1).unwrap();
    assert!(
        matches!(settled.read_current_with_policy(&request, WorthQueryOperationDeliveryPolicy::currentness(one)),
        Err(WorthQueryOperationDeliveryDenial::Currentness(worth_query_host::facade::application_entry::WorthQueryOutputCurrentnessDenial::Output(denial)))
        if matches!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))
    );
    let comparison_work = NonZeroUsize::new(
        world
            .application
            .runtime()
            .output_demand_resource_profile()
            .limits()
            .source_currentness_work(),
    )
    .expect("the installed profile admits positive source currentness work");
    let stopped_query = settled
        .read_current_with_policy(
            &request,
            WorthQueryOperationDeliveryPolicy::currentness(comparison_work).query(one, one),
        )
        .map(|_| ());
    assert!(
        matches!(
            stopped_query,
            Err(WorthQueryOperationDeliveryDenial::Query(_))
        ),
        "the declared query limit must be reached after complete currentness: {stopped_query:?}"
    );
    let delivered = settled
        .read_current(&request)
        .expect("current delivery retries the same settlement");
    assert_eq!(delivered.rows()[0].value, length(3));
    assert_eq!(planar_source_adjustment_contacts(&contact_identity), (1, 1));
    let historical = request
        .at(settled.settlement().observation())
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(historical.rows()[0].value, length(4));
    drop(historical);
    drop(delivered);
    adopt(
        &request,
        *world
            .application
            .supported_program::<RemovedFixed>()
            .unwrap()
            .owned_revision(),
    );
    assert!(matches!(
        settled.read_current(&request),
        Err(WorthQueryOperationDeliveryDenial::Action(_))
    ));
    let historical = request
        .at(settled.settlement().observation())
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(historical.rows()[0].value, length(4));
    drop(historical);

    let (settled, denial) = settled
        .close(&request)
        .err()
        .expect("live native caller capability refuses actual cleanup");
    assert!(matches!(denial, WorthQueryOperationCloseDenial::Recovery(worth_query_host::facade::product::WorthQueryProductUnpublishedRecoveryReleaseDenial::World(RuntimeWorldRecoveryDenial::CallerCapabilityLive))));
    assert_eq!(
        world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1
    );
    drop(held_partial);
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    settled
        .close(&world.application.request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, denial)| {
            panic!("released native capability permits exact cleanup: {denial:?}")
        });
    assert!(
        world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .is_empty()
    );
    assert_eq!(planar_source_adjustment_contacts(&contact_identity), (1, 1));
}

fn adopt(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        ConsumerSchema,
    >,
    revision: worth_query_decl::facade::application_program::ApplicationProgramRevision,
) {
    let programs = request.programs();
    let requirements = programs.compare(&revision).unwrap();
    let prepared = programs
        .adopt(&requirements)
        .unwrap()
        .prepare(4096)
        .unwrap();
    assert!(matches!(request.programs().publish(prepared),
        worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)));
}
