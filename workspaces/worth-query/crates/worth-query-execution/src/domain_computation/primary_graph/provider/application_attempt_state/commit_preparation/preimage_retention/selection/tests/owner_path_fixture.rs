//! Real decision facts and validated owner footprints for preimage selection.
use super::*;

pub(super) fn owner_path_fixture() -> (
    InstalledPreImageDemand,
    WorthQueryPrimaryGraphApplicationDecisionFact,
    ValidatedMutationFootprint,
    EntityId,
) {
    let world = installed_authorization_world(true);
    let account = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase.test_host_request();

            let borrowed_execution = phase
                .execution_request_for(&world.application.product_runtime)
                .expect("fixture borrows its runtime request");
            let execution = &borrowed_execution;

            world
                .application
                .select_product_branch(
                    execution,
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let field = AccountStatus::reference();
    let graph = world.application.primary_provider.graph.clone();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let read_scope = operation
        .contracts()
        .graph_reads()
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .find(|scope| {
            let WorthQueryOperationGraphReadScope::NativeProjection(scope) = scope else {
                return false;
            };
            scope.entity().semantic_key() == field.entity()
                && scope.aspect().as_str() == field.aspect()
                && scope
                    .projection()
                    .mask()
                    .paths()
                    .contains(locator.field_path())
        })
        .cloned()
        .expect("installed operation carries the exact field read scope");
    let fact = WorthQueryPrimaryGraphApplicationDecisionFact::application(
        read_scope,
        WorthQueryApplicationObservedFact::Field {
            entity_id: account.entity_id(),
            kind: account.entity_kind(),
            locator: locator.clone(),
            value: AspectValue::String(InternedString::from("open")),
        },
    );
    let footprint = world
        .application
        .with_host_advancement(|phase| {
            let borrowed_execution = phase
                .execution_request_for(&world.application.product_runtime)
                .expect("fixture borrows its runtime request");
            let execution = &borrowed_execution;

            validated_status_footprint(execution, &world, account.entity_id(), locator)
        })
        .expect("the fixture read owns its installed host call");
    let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) =
        operation.contracts().aftermath().unwrap().mechanism()
    else {
        panic!("fixture operation installs a recorded inverse");
    };
    (
        inverse.preimage_demand().clone(),
        fact,
        footprint,
        account.entity_id(),
    )
}

fn validated_status_footprint(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    entity: EntityId,
    locator: AspectFieldLocator,
) -> ValidatedMutationFootprint {
    let graph = world.application.primary_provider.graph.clone();
    let validated = graph.with_runtime_mut(request_execution, |runtime| {
        let fields = AspectFieldPatch::from(BTreeMap::from([(
            locator,
            AspectValue::String(InternedString::from("frozen")),
        )]));
        let mut transaction = {
            let transaction_validation_input = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .expect("main branch binding");
            runtime
                .begin_branch_transaction(
                    &transaction_validation_input,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("owner-admitted transaction context")
        };
        transaction
            .push_batch(
                WorkerIntentBatch::new("selection-owner-path").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields,
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
        transaction
            .validate(request_execution, runtime)
            .expect("owner validates mutation")
    });
    validated
        .mutation_footprint(Some(&()))
        .into_projected()
        .expect("explicit demand projects owner footprint")
}
