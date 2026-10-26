//! Complete paths reach the ordinary and admitted planning owners identically.
use super::*;
use crate::application_query::requirements::derive_graph_read_access_requirements_for_contract_admitted;
use crate::graph_read_access::*;
use worth_query_declaration::facade::application_query::*;
worth_query_relation!(pub AccountNext in PlanningTestSchema, Account => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_application_query!(pub(super) ClosureQuery for PlanningTestSchema, identity "planning-complete-closure", parameters ActivityParametersBinding, result ActivityResultBinding, scope Account => "Account", name "planning_complete_closure");
worth_query_application_query!(pub(super) ReverseClosureQuery for PlanningTestSchema, identity "planning-reverse-complete-closure", parameters ActivityParametersBinding, result ActivityResultBinding, scope Account => "Account", name "planning_reverse_complete_closure");
pub(super) fn definition<Q: ApplicationQueryMarkerIdentity<PlanningTestSchema> + 'static>(
    reference: ApplicationQueryReference<
        PlanningTestSchema,
        Q,
        ActivityParameters,
        ActivityResult,
        Account,
    >,
    reverse: bool,
) -> ApplicationQueryDefinition<PlanningTestSchema, Q, ActivityParameters, ActivityResult, Account>
where
    Q::ResultBinding:
        worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
            Value = ActivityResult,
        >,
{
    let field = ApplicationQueryResultFieldRef::<Q, AccountIdSlot, _, _, _, _, _, _, _, _>::new(
        "account_id",
        AccountId::reference(),
    );
    let shape = ApplicationQueryResultShapeBuilder::<
        PlanningTestSchema,
        Q,
        Account,
        ActivityResult,
        Q::ResultBinding,
    >::new(Account::reference())
    .field(field)
    .build();
    let path = ApplicationQueryRootPath::from(Account::reference());
    let path = if reverse {
        path.reverse_closure(AccountNext::reference())
    } else {
        path.forward_closure(AccountNext::reference())
    };
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(path)
        .build()
        .unwrap()
}
#[test]
fn complete_closure_installed_requirements_are_unbounded_and_currently_supported() {
    check(ClosureQuery::reference(), false);
    check(ReverseClosureQuery::reference(), true);
}
fn check<Q: ApplicationQueryMarkerIdentity<PlanningTestSchema> + 'static>(
    reference: ApplicationQueryReference<
        PlanningTestSchema,
        Q,
        ActivityParameters,
        ActivityResult,
        Account,
    >,
    reverse: bool,
) where
    Q::ResultBinding:
        worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
            Value = ActivityResult,
        >,
{
    let schema = installed_schema();
    let query = schema.certification_query(reference).unwrap();
    let relation = worth_query_installation::facade::WorthQueryReadGraphPlanningContract::relation(
        query.read_graph(),
        0,
    )
    .unwrap();
    assert!(relation.complete_closure);
    assert_eq!(
        relation.direction,
        if reverse {
            worth_query_installation::facade::WorthQueryReadGraphRelationDirection::Reverse
        } else {
            worth_query_installation::facade::WorthQueryReadGraphRelationDirection::Forward
        }
    );
    let parameters =
        admit_application_query_parameters(&query, ApplicationQueryParameterSet::new()).unwrap();
    let ordinary = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );
    let actual = derive_graph_read_access_requirements_for_contract_admitted(
        query.read_family_binding().planning_contract(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
        test_canonical_budget(),
        &mut |_, _| Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(ordinary, actual);
    for kind in [
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
        WorthQueryGraphReadAccessRequirementKind::VisitedSet,
    ] {
        let row = ordinary.rows().iter().find(|r| r.kind() == &kind).unwrap();
        assert_eq!(
            row.traversal_operator(),
            Some(&WorthQueryGraphReadTraversalOperator::CompleteClosure)
        );
        assert_eq!(
            row.complexity_contract(),
            &WorthQueryGraphReadAccessComplexityContract::CompleteTraversalWorkset
        );
        assert_eq!(
            row.memory_estimate_basis(),
            &WorthQueryGraphReadAccessMemoryEstimateBasis::DiscoveredWorkset
        );
    }
    let inventory = WorthQueryGraphIndexInventory::from_current_runtime_support();
    let admitted_inventory = WorthQueryGraphIndexInventory::from_rows_admitted(
        WorthQueryGraphIndexInventory::current_runtime_support_rows(),
        &mut |_, _| Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(inventory, admitted_inventory);
    let report = match_graph_index_inventory_for_requirements(&ordinary, &inventory);
    assert!(
        report
            .matches()
            .iter()
            .all(|m| m.outcome().as_str() == "exact_match"),
        "{report:?}"
    );
    let estimate = estimate_graph_read_access_cost(
        &ordinary,
        WorthQueryGraphReadCostEvidence::unknown_conservative(),
    );
    assert_eq!(estimate.status().as_str(), "unknown_conservative");
    assert_eq!(
        estimate.complexity_contract().as_str(),
        "broad_traversal_candidate"
    );
}
