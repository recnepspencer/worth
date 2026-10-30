//! Genuine bound query completion, current capability admission and Native writes.
use super::*;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        admit_touch_account_capability,
        capability::{CapabilityTouchInput, CapabilityTouchOperation, TouchAccountCapability},
        diagnostic_comparison::{
            DiagnosticParametersBinding, DiagnosticResult, DiagnosticResultBinding,
        },
        publish_relational_mutation, status_parameter, Account, AccountPolicy, AccountStatus,
        AccountSummaryParameters, AuthorizationWorld, ExternalMapping, IdentityBinding,
        IdentityExecutionSchema, Principal,
    },
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryFieldScope, ApplicationQueryParameterSet,
    },
    application_schema::{NoApplicationUnit, ReadWrite, U64ApplicationValueBinding},
};
use worth_relational::facade::identity::EntityId;

type Scope = ApplicationQueryFieldScope<
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountStatus,
    String,
    ReadWrite,
    NoApplicationUnit,
>;

pub(in super::super) trait DiagnosticBinding:
    ApplicationQueryBinding<
    IdentityExecutionSchema,
    Input = AccountSummaryParameters,
    InputBinding = DiagnosticParametersBinding,
    ParameterBinding = DiagnosticParametersBinding,
    ResultBinding = DiagnosticResultBinding,
    ScopeBinding = Scope,
    PrincipalBinding = IdentityBinding,
    Mapping = ExternalMapping,
    Principal = Principal,
    PrincipalIdentity = u64,
    PrincipalIdentityBinding = U64ApplicationValueBinding,
>
{
}
impl<B> DiagnosticBinding for B where
    B: ApplicationQueryBinding<
        IdentityExecutionSchema,
        Input = AccountSummaryParameters,
        InputBinding = DiagnosticParametersBinding,
        ParameterBinding = DiagnosticParametersBinding,
        ResultBinding = DiagnosticResultBinding,
        ScopeBinding = Scope,
        PrincipalBinding = IdentityBinding,
        Mapping = ExternalMapping,
        Principal = Principal,
        PrincipalIdentity = u64,
        PrincipalIdentityBinding = U64ApplicationValueBinding,
    >
{
}

pub(super) type Capability = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    TouchAccountCapability,
    CapabilityTouchOperation,
    CapabilityTouchInput,
>;

pub(in super::super) fn capability(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
) -> Capability {
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            admit_touch_account_capability(&phase.test_host_request(), world, &principal, request)
                .unwrap()
        })
        .unwrap()
}

pub(in super::super) fn read<B: DiagnosticBinding>(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
    status: &str,
    retain: bool,
) -> (WorthQueryApplicationQueryObservation, Vec<DiagnosticResult>) {
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let scope = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let query = world
                .application
                .installed_schema()
                .installed_query_binding::<B>()
                .unwrap()
                .into_query();
            let capability = admit_touch_account_capability(
                &phase.test_host_request(),
                world,
                &principal,
                request,
            )
            .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let plan = selected
                .admit_governed_application_query(
                    &phase,
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), status.to_owned())
                        .unwrap(),
                    WorthQueryProductQueryControls::new(std::num::NonZeroUsize::MIN, request),
                )
                .unwrap();
            let plan = if retain {
                plan.retain_projection_for_inspection()
            } else {
                plan
            };
            let result = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap();
            let rows = result.rows().to_vec();
            let original = result
                .into_admitted_disclosed()
                .into_output_demand_source()
                .retain_inspection_observation()
                .unwrap();
            (original, rows)
        })
        .unwrap()
}

pub(super) fn compare<'o, B: DiagnosticBinding>(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
    before: &'o WorthQueryApplicationQueryObservation,
    after: &'o WorthQueryApplicationQueryObservation,
) -> Result<WorthQueryComparedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial> {
    let diagnostic = capability(world, external, request);
    world
        .application
        .inspect_application_observation::<B>(external, request, after)
        .why_changed(before)
        .reference_only()
        .execute_with_diagnostic(diagnostic)
}

pub(super) fn write(
    world: &AuthorizationWorld,
    field: (&str, &str, &str),
    entity: EntityId,
    value: worth_foundational::facade::AspectValue,
) {
    use worth_relational::facade::transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let locator = graph
        .layout
        .field_locator(field.0, field.1, field.2)
        .unwrap()
        .clone();
    world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            publish_relational_mutation(
                &custody.borrowed_request_for_test(),
                world,
                WorkerIntentBatch::new("comparison-native-edit").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: AspectFieldPatch::from_locator(locator, value),
                    }),
                )),
            );
        })
        .unwrap();
}

/// Native readmission validates this descriptive clear against the actual contract.
pub(super) fn clear_optional_field(
    world: &AuthorizationWorld,
    field: (&str, &str, &str),
    entity: EntityId,
) {
    use worth_foundational::facade::{
        PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
    };
    use worth_relational::facade::transactions::{
        ApplyEntityAspectPatchIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let locator = graph
        .layout
        .field_locator(field.0, field.1, field.2)
        .unwrap();
    assert_eq!(locator.field_path().fields().len(), 1);
    let contract = graph
        .layout
        .aspect_contract(field.0, locator.aspect().aspect_key())
        .unwrap();
    let key = locator.field_path().fields()[0].clone();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::PatchFields {
        basis: PortableAspectContractBasis::from_contract(contract),
        selected_fields: vec![key.clone()],
        field_sets: vec![],
        field_clears: vec![key],
    }]);
    world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            publish_relational_mutation(
                &custody.borrowed_request_for_test(),
                world,
                WorkerIntentBatch::new("comparison-native-optional-clear").push(
                    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
                        ApplyEntityAspectPatchIntent {
                            entity_id: entity,
                            aspect_patch: patch,
                        },
                    )),
                ),
            );
        })
        .unwrap();
}

pub(super) fn account(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    key: &str,
) -> EntityId {
    world.application.with_host_advancement(|phase| world.selected_product(&phase.test_host_request())
        .resolve_entity(&phase, crate::domain_computation::primary_graph::tests::fixture::AccountIdentity::reference(),
            key.to_owned(), request, WorthQueryPrincipalResolutionMode::Ordinary).unwrap().entity_id()).unwrap()
}

pub(super) fn activity(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    key: &str,
) -> EntityId {
    world.application.with_host_advancement(|phase| world.selected_product(&phase.test_host_request())
        .resolve_entity(&phase, crate::domain_computation::primary_graph::tests::fixture::ActivityIdentity::reference(),
            key.to_owned(), request, WorthQueryPrincipalResolutionMode::Ordinary).unwrap().entity_id()).unwrap()
}
