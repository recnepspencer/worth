use super::*;

pub(super) fn query_definition(
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

pub(super) fn installed_schema(
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
