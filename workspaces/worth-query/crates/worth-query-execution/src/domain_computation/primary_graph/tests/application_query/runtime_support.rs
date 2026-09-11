use worth_query_admission::facade::{
    application_query::{admit_application_query_parameters, WorthQueryApplicationQueryLane},
    graph_read_access::{
        WorthQueryGraphIndexInventory, WorthQueryGraphIndexLifecycleClass,
        WorthQueryGraphIndexLifecycleOwner, WorthQueryGraphIndexPosture,
        WorthQueryGraphIndexSupportState, WorthQueryGraphReadAccessRequirementKind,
        WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadBudget,
        WorthQueryGraphReadPlanReviewDenialKind,
    },
};
use worth_query_admission::integration::{
    derive_graph_read_access_requirements_for_contract, review_graph_read_access,
};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryParameterSet, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_relational::facade::schema::RelationalSchemaRegistry;

use super::super::fixture::{installed_authorization_world, live_account_parameters};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::{
    application_query::primary_graph_support_inventory, schema_layout::WorthQueryPrimaryGraphLayout,
};

#[test]
fn live_support_requires_the_exact_contract_and_both_equality_indexes() {
    let world = installed_authorization_world(true);
    let query = super::installed_live_query(&world);
    let live = query.live().expect("fixture installs live meaning");
    let parameters =
        admit_application_query_parameters(&query, live_account_parameters("account-1")).unwrap();
    let requirements = derive_graph_read_access_requirements_for_contract(
        query.read_family_binding().planning_contract(),
        WorthQueryApplicationQueryLane::Live,
        4,
        parameters.identity(),
        query.canonical_work_policy().admission_planning(),
    )
    .expect("the live fixture fits its installed canonical budget");
    let graph = world.application.runtime.primary_graph().unwrap();

    let supported = primary_graph_support_inventory(
        &graph.layout,
        query.continuation(),
        Some(live),
        &requirements,
    );
    let row = supported
        .row_for_requirement_kind(&WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport)
        .expect("exact installed live contract and indexes must emit support");
    assert_eq!(
        row.lifecycle_owner(),
        &WorthQueryGraphIndexLifecycleOwner::QueryRuntime
    );
    assert_eq!(
        row.lifecycle_class(),
        &WorthQueryGraphIndexLifecycleClass::RuntimeMaintained
    );
    assert_eq!(row.posture(), &WorthQueryGraphIndexPosture::Verified);
    assert_eq!(
        row.support_state(),
        &WorthQueryGraphIndexSupportState::Available
    );

    assert_live_support_absent(primary_graph_support_inventory(
        &graph.layout,
        query.continuation(),
        None,
        &requirements,
    ));
    assert_live_support_absent(primary_graph_support_inventory(
        &lower_layout(missing_scope_index::MissingScopeIndexSchema::declaration().unwrap()),
        query.continuation(),
        Some(live),
        &requirements,
    ));
    assert_live_support_absent(primary_graph_support_inventory(
        &lower_layout(missing_target_index::MissingTargetIndexSchema::declaration().unwrap()),
        query.continuation(),
        Some(live),
        &requirements,
    ));
}

#[test]
fn provider_mechanism_deletion_mints_no_support_row() {
    let world = installed_authorization_world(true);
    let query = super::installed_nested_query(&world);
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(super::status_parameter(), "open".to_string())
            .expect("fixture query parameter must encode"),
    )
    .unwrap();
    let requirements = derive_graph_read_access_requirements_for_contract(
        query.read_family_binding().planning_contract(),
        WorthQueryApplicationQueryLane::OneShot,
        10,
        parameters.identity(),
        query.canonical_work_policy().admission_planning(),
    )
    .expect("the one-shot fixture fits its installed canonical budget");
    let graph = world.application.runtime.primary_graph().unwrap();
    let hostile =
        lower_layout(missing_scope_index::MissingScopeIndexSchema::declaration().unwrap());

    for kind in [
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency,
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
    ] {
        let isolated = isolated_requirement(&requirements, kind.clone());
        assert_support_present(primary_graph_support_inventory(
            &graph.layout,
            query.continuation(),
            query.live(),
            &isolated,
        ));
        assert_support_absent(primary_graph_support_inventory(
            &hostile,
            query.continuation(),
            query.live(),
            &isolated,
        ));
    }
}

#[test]
fn continuation_ordering_requires_the_installed_seek_mechanism() {
    let world = installed_authorization_world(true);
    let query = super::installed_live_query(&world);
    let parameters =
        admit_application_query_parameters(&query, live_account_parameters("account-1")).unwrap();
    let requirements = derive_graph_read_access_requirements_for_contract(
        query.read_family_binding().planning_contract(),
        WorthQueryApplicationQueryLane::Continuation,
        4,
        parameters.identity(),
        query.canonical_work_policy().admission_planning(),
    )
    .expect("the continuation fixture fits its installed canonical budget");
    let ordering = isolated_requirement(
        &requirements,
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
    );
    let graph = world.application.runtime.primary_graph().unwrap();

    assert_support_present(primary_graph_support_inventory(
        &graph.layout,
        query.continuation(),
        query.live(),
        &ordering,
    ));
    assert_support_absent(primary_graph_support_inventory(
        &graph.layout,
        None,
        query.live(),
        &ordering,
    ));
}

#[test]
fn query_runtime_mechanism_deletion_denies_plan_review() {
    let world = installed_authorization_world(true);
    let query = super::installed_nested_query(&world);
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(super::status_parameter(), "open".to_string())
            .expect("fixture query parameter must encode"),
    )
    .unwrap();
    let requirements = derive_graph_read_access_requirements_for_contract(
        query.read_family_binding().planning_contract(),
        WorthQueryApplicationQueryLane::OneShot,
        10,
        parameters.identity(),
        query.canonical_work_policy().admission_planning(),
    )
    .expect("the nested fixture fits its installed canonical budget");
    let graph = world.application.runtime.primary_graph().unwrap();

    for kind in [
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
        WorthQueryGraphReadAccessRequirementKind::VisitedSet,
        WorthQueryGraphReadAccessRequirementKind::ProofSupport,
        WorthQueryGraphReadAccessRequirementKind::ResultBuffer,
        WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle,
    ] {
        let isolated = isolated_requirement(&requirements, kind.clone());
        let inventory = primary_graph_support_inventory(
            &graph.layout,
            query.continuation(),
            query.live(),
            &isolated,
        );
        assert!(
            review_graph_read_access(isolated.clone(), inventory.clone(), application_budget(),)
                .is_admitted(),
            "the installed runtime must support {kind:?}"
        );
        let deleted = WorthQueryGraphIndexInventory::from_rows(
            inventory
                .rows()
                .iter()
                .filter(|row| row.requirement_kind() != &kind)
                .cloned()
                .collect(),
        );
        let review = review_graph_read_access(isolated, deleted, application_budget());
        assert_eq!(
            review.denial().map(|denial| denial.kind()),
            Some(WorthQueryGraphReadPlanReviewDenialKind::UnsupportedGraphIndexSupport),
            "deleting {kind:?} must deny rather than invent fallback support"
        );
    }
}

fn application_budget() -> WorthQueryGraphReadBudget {
    let runtime_default = WorthQueryGraphReadBudget::inline_ephemeral_default();
    WorthQueryGraphReadBudget::bounded(
        runtime_default.max_inline_index_bytes(),
        runtime_default.max_inline_result_bytes().saturating_mul(10),
        10_000,
    )
}

fn isolated_requirement(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
    kind: WorthQueryGraphReadAccessRequirementKind,
) -> WorthQueryGraphReadAccessRequirementSet {
    let row = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &kind)
        .unwrap_or_else(|| panic!("fixture must derive {kind:?}"))
        .clone();
    WorthQueryGraphReadAccessRequirementSet::new(
        *requirements.read_graph_digest(),
        *requirements.access_shape_digest(),
        *requirements.selectivity_shape_digest(),
        vec![row],
        worth_foundational::facade::CanonicalDigestWorkBudget::new(64, 16 * 1024)
            .expect("the isolated-requirement test budget is nonzero"),
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
    )
    .expect("the isolated requirement fits its canonical budget")
}

fn assert_support_present(inventory: WorthQueryGraphIndexInventory) {
    assert_eq!(
        inventory.rows().len(),
        1,
        "the installed mechanism must emit its one isolated support row"
    );
}

fn assert_support_absent(inventory: WorthQueryGraphIndexInventory) {
    assert!(
        inventory.rows().is_empty(),
        "a missing provider mechanism must mint no support row"
    );
}

fn assert_live_support_absent(
    inventory: worth_query_admission::facade::graph_read_access::WorthQueryGraphIndexInventory,
) {
    assert!(
        inventory
            .row_for_requirement_kind(
                &WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport
            )
            .is_none(),
        "an absent live prerequisite must mint no verified support row"
    );
}

fn lower_layout<Schema: ApplicationSchema>(
    declaration: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration<
        Schema,
    >,
) -> WorthQueryPrimaryGraphLayout {
    let owner = declaration.erased().owner().to_string();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        owner,
        declaration.erased().major(),
        declaration.erased().minor(),
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap();
    let (runtime, _) = installation.into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    WorthQueryPrimaryGraphLayout::lower(
        installed.installed_declaration(),
        installed.native_contracts(),
        &RelationalSchemaRegistry::new(),
    )
    .unwrap()
    .0
}

macro_rules! hostile_live_index_schema {
    ($module:ident, $schema:ident, $owner:ident, $scope_equality:ident, $target_equality:ident) => {
        mod $module {
            use worth_query_declaration::{
                worth_query_application_schema, worth_query_aspect, worth_query_entity,
                worth_query_field,
            };

            worth_query_application_schema! {
                pub schema $schema {
                    owner: $owner,
                    version: (1, 0),
                    members: |schema| {
                        schema
                            .entity(Account::reference())
                            .entity(Activity::reference())
                            .aspect(Account::reference(), AccountPolicy::reference())
                            .field(Account::reference(), AccountIdentity::reference())
                            .aspect(Activity::reference(), ActivityFacts::reference())
                            .field(Activity::reference(), ActivityIdentity::reference())
                    }
                }
            }
            worth_query_entity!(pub Account for $schema);
            worth_query_entity!(pub Activity for $schema);
            worth_query_aspect!(pub AccountPolicy for $schema, Account; identity = AspectIdentity(0x91611034), revision = AspectContractRevision(1),);
            worth_query_field!(
                pub AccountIdentity for $schema, Account, AccountPolicy:
                String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, $scope_equality
            );
            worth_query_aspect!(pub ActivityFacts for $schema, Activity; identity = AspectIdentity(0x91611035), revision = AspectContractRevision(1),);
            worth_query_field!(
                pub ActivityIdentity for $schema, Activity, ActivityFacts:
                String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, $target_equality
            );
        }
    };
}

hostile_live_index_schema!(
    missing_scope_index,
    MissingScopeIndexSchema,
    missing_scope_index_fixture,
    no_equality,
    equality
);
hostile_live_index_schema!(
    missing_target_index,
    MissingTargetIndexSchema,
    missing_target_index_fixture,
    equality,
    no_equality
);
