use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDependencyCeiling, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryLiveResourceContract,
    ApplicationQueryOrderingDirection, ApplicationQueryParameterRef, ApplicationQueryParameterSet,
    ApplicationQueryReference, ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_portable_type, worth_query_relation,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPreparedReadGraphPlanningContract,
};

use crate::facade::{
    application_query::{
        admit_application_query_parameters, derive_graph_read_access_requirements_for_contract,
        WorthQueryApplicationQueryLane,
    },
    graph_read_access::{
        WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementSet,
        WorthQueryGraphReadOrderingPosture, WorthQueryGraphReadResultPressure,
    },
};

#[path = "tests/alternate_source_digest.rs"]
mod alternate_source_digest;
#[path = "tests/installed_contract.rs"]
mod installed_contract;
#[path = "tests/live_lane.rs"]
mod live_lane;
#[path = "tests/parameter_canonical_basis.rs"]
mod parameter_canonical_basis;
#[path = "tests/planning_variations.rs"]
mod planning_variations;
#[path = "tests/without_traversal.rs"]
mod without_traversal;
use alternate_source_digest::AlternateSourceDigestGraph;
use without_traversal::WithoutTraversalGraph;

fn test_canonical_budget() -> CanonicalDigestWorkBudget {
    CanonicalDigestWorkBudget::new(4_096, 1024 * 1024)
        .expect("the application-query test canonical budget is nonzero")
}

fn admitted_requirements(
    graph: &impl WorthQueryPreparedReadGraphPlanningContract,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    selectivity_binding_digest: &CanonicalDigestId,
) -> WorthQueryGraphReadAccessRequirementSet {
    derive_graph_read_access_requirements_for_contract(
        graph,
        lane,
        maximum_result_count,
        selectivity_binding_digest,
        test_canonical_budget(),
    )
    .expect("the application-query fixture fits its installed canonical budget")
}

worth_query_application_schema! {
    pub schema PlanningTestSchema {
        owner: application_query_planning_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Account::reference())
                .entity(Activity::reference())
                .aspect(Account::reference(), AccountFacts::reference())
                .aspect(Activity::reference(), ActivityFacts::reference())
                .field(Account::reference(), AccountId::reference())
                .field(Activity::reference(), Sequence::reference())
                .relation(
                    AccountActivity::reference(),
                    Account::reference(),
                    Activity::reference(),
                )
                .effect(live_lane::PlanningLiveEffect::reference())
                .application_query(query_definition())
        }
    }
}

worth_query_entity!(pub Account for PlanningTestSchema);
worth_query_entity!(pub Activity for PlanningTestSchema);
worth_query_aspect!(pub AccountFacts for PlanningTestSchema, Account; identity = AspectIdentity(0x91611019), revision = AspectContractRevision(1),);
worth_query_aspect!(pub ActivityFacts for PlanningTestSchema, Activity; identity = AspectIdentity(0x9161101a), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountId for PlanningTestSchema, Account, AccountFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub Sequence for PlanningTestSchema, Activity, ActivityFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_only, equality
);
worth_query_relation!(
    pub AccountActivity in PlanningTestSchema, Account => Activity; integrity = same_context_unbounded_retain_dangling);

struct ActivityParameters;
struct ActivityResult;
struct AccountParameter;
struct AccountIdSlot;
struct SequenceSlot;
struct ActivitySlot;

worth_query_declaration::worth_query_structured_value_binding!(
    ActivityParametersBinding for ActivityParameters {
        identity: "worth.query.test.admission.activity-parameters.v1"
    }
);
worth_query_declaration::worth_query_structured_value_binding!(
    ActivityResultBinding for ActivityResult {
        identity: "worth.query.test.admission.activity-result.v1"
    }
);
worth_query_declaration::worth_query_structured_value_binding!(
    ActivityNestedResultBinding for () {
        identity: "worth.query.test.admission.activity-nested-result.v1"
    }
);

worth_query_portable_type!(ActivityResult => "worth.query.test.admission.activity-result.v1");
worth_query_portable_type!(AccountIdSlot => "worth.query.test.admission.account-id-slot.v1");
worth_query_portable_type!(SequenceSlot => "worth.query.test.admission.sequence-slot.v1");
worth_query_portable_type!(ActivitySlot => "worth.query.test.admission.activity-slot.v1");

worth_query_application_query!(
    ActivityQuery for PlanningTestSchema,
    identity "worth.query.test.admission.activity-query.v1",
    parameters ActivityParametersBinding,
    result ActivityResultBinding,
    scope Account => "Account",
    name "account_activity"
);

fn query_reference() -> ApplicationQueryReference<
    PlanningTestSchema,
    ActivityQuery,
    ActivityParameters,
    ActivityResult,
    Account,
> {
    ActivityQuery::reference()
}

fn account_parameter() -> ApplicationQueryParameterRef<
    ActivityQuery,
    AccountParameter,
    worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("account")
}

#[test]
fn traversal_alone_adds_proof_support_and_wide_result_pressure() {
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let without_traversal = admitted_requirements(
        &WithoutTraversalGraph::new(query.read_graph()),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );

    assert!(
        !without_traversal.contains_kind(&WorthQueryGraphReadAccessRequirementKind::ProofSupport)
    );
    let result_buffer = without_traversal
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::ResultBuffer)
        .unwrap();
    assert_eq!(
        result_buffer.result_pressure(),
        Some(&WorthQueryGraphReadResultPressure::Detail)
    );
}

#[test]
fn parameter_values_change_selectivity_identity_without_changing_requirement_rows() {
    let query = installed_query();
    let left = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let right = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 8_u64)
            .unwrap(),
    )
    .unwrap();
    let left_requirements = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        left.identity(),
    );
    let right_requirements = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        right.identity(),
    );

    assert_eq!(left_requirements.rows(), right_requirements.rows());
    assert_eq!(
        left_requirements.access_shape_digest(),
        right_requirements.access_shape_digest()
    );
    assert_ne!(
        left_requirements.selectivity_shape_digest(),
        right_requirements.selectivity_shape_digest()
    );
    assert_ne!(left_requirements.digest(), right_requirements.digest());
}

#[test]
fn parameter_binding_identity_is_canonical_and_value_sensitive() {
    let query = installed_query();
    let left = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let equivalent = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let changed = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 8_u64)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(left.identity(), equivalent.identity());
    assert_ne!(left.identity(), changed.identity());
}

#[test]
fn alternate_source_digest_does_not_split_one_installed_contract() {
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let application = admitted_requirements(
        query.read_family_binding().planning_contract(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );
    let alternate = AlternateSourceDigestGraph(query.read_family_binding().planning_contract());
    let equivalent = admitted_requirements(
        &alternate,
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );

    assert_eq!(application, equivalent);
}

fn installed_query() -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    PlanningTestSchema,
    ActivityQuery,
    ActivityParameters,
    ActivityResult,
    Account,
> {
    installed_schema()
        .application_query(query_reference())
        .unwrap()
}

fn query_definition(
) -> worth_query_declaration::facade::application_query::ApplicationQueryDefinition<
    PlanningTestSchema,
    ActivityQuery,
    ActivityParameters,
    ActivityResult,
    Account,
> {
    let sequence =
        ApplicationQueryResultFieldRef::<ActivityQuery, SequenceSlot, _, _, _, _, _, _, _, _>::new(
            "sequence",
            Sequence::reference(),
        );
    let account_id = ApplicationQueryResultFieldRef::<
        ActivityQuery,
        AccountIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("account_id", AccountId::reference());
    let activity_relation = ApplicationQueryResultRelationRef::<
        ActivityQuery,
        ActivitySlot,
        _,
        _,
        _,
        _,
        ForwardResultTraversal,
        ManyResults,
    >::forward_many("activity", AccountActivity::reference());
    let nested = ApplicationQueryResultShapeBuilder::<
        PlanningTestSchema,
        ActivityQuery,
        Activity,
        (),
        ActivityNestedResultBinding,
    >::new(Activity::reference())
    .field(sequence);
    let shape = ApplicationQueryResultShapeBuilder::<
        PlanningTestSchema,
        ActivityQuery,
        Account,
        ActivityResult,
        ActivityResultBinding,
    >::new(Account::reference())
    .field(account_id)
    .relation(activity_relation, nested)
    .build();
    ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .parameter(account_parameter())
        .where_equal(AccountId::reference(), account_parameter())
        .order_by(sequence, ApplicationQueryOrderingDirection::Ascending)
        .continue_by(activity_relation)
        .live_by::<Activity, live_lane::PlanningLiveCause, _, _, _, _, _, _, _, _>(
            account_id,
            sequence,
            ApplicationQueryLiveResourceContract::bounded(4, 2_048, 4_096),
        )
        .build()
        .unwrap()
}

fn installed_schema(
) -> worth_query_installation::facade::WorthQueryInstalledApplicationSchema<PlanningTestSchema> {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "application_query_planning_test",
        1,
        0,
    ))
    .application_schema(PlanningTestSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(PlanningTestSchema::declaration().unwrap())
    .unwrap()
}
use worth_foundational::facade::{CanonicalDigestId, CanonicalDigestWorkBudget};
