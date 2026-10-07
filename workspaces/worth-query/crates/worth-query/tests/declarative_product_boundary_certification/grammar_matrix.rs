use super::product_support as fixture;

fn type_exists<T>() {}

#[test]
fn grammar_read_journey_executes() {
    use worth_query::facade::read::{
        current, declare, WorthQueryReadNextAction, WorthQueryReadOutcome, WorthQueryReadStop,
    };
    let mut workspace = fixture::workspace("grammar-read");
    let outcome: WorthQueryReadOutcome = declare(fixture::identity_detail)
        .expect("read should declare")
        .using(current())
        .run(&mut workspace);
    assert!(outcome.completed().is_some());
    type_exists::<WorthQueryReadStop>();
    type_exists::<WorthQueryReadNextAction>();
}

#[test]
fn grammar_aggregate_journey_executes() {
    use worth_query::facade::aggregate::{
        current, declare, WorthQueryCountDeclarationStop, WorthQueryCountOutcome,
        WorthQueryReadNextAction,
    };
    let mut workspace = fixture::workspace("grammar-aggregate");
    let outcome: WorthQueryCountOutcome = declare(fixture::identity_collection)
        .expect("count should declare")
        .using(current())
        .run(&mut workspace);
    assert!(outcome.completed().is_some());
    type_exists::<WorthQueryCountDeclarationStop>();
    type_exists::<WorthQueryReadNextAction>();
}

#[test]
fn grammar_live_journey_executes() {
    use worth_query::facade::live::{
        current, declare, WorthQueryLiveDeclarationStop, WorthQueryLiveOpenOutcome,
        WorthQueryManagedLiveCloseOutcome, WorthQueryReadNextAction,
    };
    let mut workspace = fixture::workspace("grammar-live");
    let opened: WorthQueryLiveOpenOutcome = declare("grammar.live", fixture::identity_collection)
        .expect("live should declare")
        .using(current())
        .open(&mut workspace);
    let handle = match opened {
        WorthQueryLiveOpenOutcome::Opened(completion) => completion.into_handle(),
        WorthQueryLiveOpenOutcome::Stopped(stop) => panic!("live stopped: {:?}", stop.source()),
    };
    assert!(matches!(
        handle.close(&mut workspace),
        WorthQueryManagedLiveCloseOutcome::Closed(_)
    ));
    type_exists::<WorthQueryLiveDeclarationStop>();
    type_exists::<WorthQueryReadNextAction>();
}

#[test]
fn grammar_history_journey_executes() {
    use worth_query::facade::history::{
        at, declare, WorthQueryHistoricalNextAction, WorthQueryHistoricalOutcome,
        WorthQueryHistoricalStop,
    };
    let mut workspace = fixture::workspace("grammar-history");
    let context = at(&workspace).expect("the fixture requires an open workspace");
    let outcome: WorthQueryHistoricalOutcome = declare(fixture::identity_detail)
        .expect("history should declare")
        .retained_snapshot()
        .using(context)
        .run(&mut workspace);
    assert!(outcome.completed().is_some());
    type_exists::<WorthQueryHistoricalStop>();
    type_exists::<WorthQueryHistoricalNextAction>();
}

#[test]
fn grammar_comparison_journey_executes() {
    use worth_query::facade::comparison::{
        between, declare, WorthQueryComparisonNextAction, WorthQueryComparisonOutcome,
        WorthQueryComparisonStop, WorthQuerySessionLabel,
    };
    let mut left = fixture::workspace("grammar-comparison-left");
    let mut right = fixture::workspace("grammar-comparison-right");
    let context = between(
        &left,
        WorthQuerySessionLabel::scoped_strs("grammar", ["left"]).unwrap(),
        &right,
        WorthQuerySessionLabel::scoped_strs("grammar", ["right"]).unwrap(),
    )
    .expect("comparison basis should admit");
    let outcome: WorthQueryComparisonOutcome = declare(fixture::identity_detail)
        .expect("comparison should declare")
        .diff()
        .using(context)
        .run((&mut left, &mut right));
    assert!(outcome.completed().is_some());
    type_exists::<WorthQueryComparisonStop>();
    type_exists::<WorthQueryComparisonNextAction>();
}

#[test]
fn grammar_preview_journey_executes() {
    use worth_query::facade::preview::{
        declare, read_only, WorthQueryPreviewJourneyOutcome, WorthQuerySessionLabel,
        WorthQueryWorkflowNextAction, WorthQueryWorkflowStop,
    };
    let mut workspace = fixture::workspace("grammar-preview");
    let label = WorthQuerySessionLabel::scoped_strs("grammar", ["preview"]).unwrap();
    let context = read_only(&workspace, label.clone()).expect("preview context should admit");
    let outcome: WorthQueryPreviewJourneyOutcome =
        declare(label).using(context).open_and_close(&mut workspace);
    assert!(outcome.read_only_completion().is_some());
    type_exists::<WorthQueryWorkflowStop>();
    type_exists::<WorthQueryWorkflowNextAction>();
}

#[test]
fn grammar_mutation_journey_executes() {
    use worth_query::facade::mutation::{
        authoritative, WorthQueryMutationNextAction, WorthQueryMutationOutcome,
        WorthQueryMutationStop,
    };
    let mut workspace = fixture::workspace("grammar-mutation");
    let context = authoritative(&workspace).expect("mutation authority should admit");
    let outcome: WorthQueryMutationOutcome = fixture::mutation("grammar-mutation")
        .using(context)
        .run(&mut workspace);
    assert!(outcome.completed().is_some());
    type_exists::<WorthQueryMutationStop>();
    type_exists::<WorthQueryMutationNextAction>();
}

#[test]
fn grammar_workflow_journey_executes() {
    use worth_query::facade::workflow::{
        declare, preview, WorthQuerySessionLabel, WorthQueryWorkflowNextAction,
        WorthQueryWorkflowOutcome, WorthQueryWorkflowStop,
    };
    let mut workspace = fixture::workspace("grammar-workflow");
    let label = WorthQuerySessionLabel::scoped_strs("grammar", ["workflow"]).unwrap();
    let context = preview(&workspace, label.clone()).expect("workflow context should admit");
    let outcome: WorthQueryWorkflowOutcome = declare(label, fixture::mutation("workflow"))
        .using(context)
        .run(&mut workspace);
    assert!(outcome.completed().is_some());
    type_exists::<WorthQueryWorkflowStop>();
    type_exists::<WorthQueryWorkflowNextAction>();
}

#[test]
fn grammar_inspection_journey_executes() {
    use worth_query::facade::inspection::{
        declare, inspection_basis, WorthQueryInspectionNextAction, WorthQueryInspectionOutcome,
        WorthQueryInspectionStop,
    };
    use worth_query::facade::read::{current, declare as declare_read};
    let mut workspace = fixture::workspace("grammar-inspection");
    let completion = declare_read(fixture::identity_detail)
        .expect("read should declare")
        .using(current())
        .run(&mut workspace)
        .into_result()
        .expect("read should complete");
    let outcome: WorthQueryInspectionOutcome = declare(&completion)
        .using(inspection_basis(fixture::inspection_basis("grammar")))
        .run(&workspace);
    assert!(outcome.settled().is_some());
    type_exists::<WorthQueryInspectionStop>();
    type_exists::<WorthQueryInspectionNextAction>();
}

#[test]
fn grammar_domain_journey_executes() {
    use worth_query::facade::domain::{
        preview, WorthQueryInstalledDomainWorkflowOutcome, WorthQuerySessionLabel,
        WorthQueryWorkflowNextAction, WorthQueryWorkflowStop,
    };
    let mut workspace = crate::support::installed_domain::workspace("grammar-domain");
    let handle = workspace
        .domain(crate::support::installed_domain::PublicInstalledDomain)
        .unwrap();
    let label = WorthQuerySessionLabel::scoped_strs("grammar", ["domain"]).unwrap();
    let context = preview(&workspace, label.clone()).expect("domain context should admit");
    let outcome: WorthQueryInstalledDomainWorkflowOutcome = handle
        .mutation(|mutation| {
            mutation
                .set_aspect(
                    worth_query::facade::domain::WorthQueryAspectTouch::from_authoring_ingress_text(
                        "identity.id",
                    )?,
                    worth_query::facade::domain::WorthQueryAuthoredAspectValue::string(
                        "grammar-domain",
                    ),
                )
                .build_insert("Task")
        })
        .unwrap()
        .workflow(label)
        .using(context)
        .run(&mut workspace)
        .unwrap();
    assert!(outcome.completed().is_some());
    type_exists::<WorthQueryWorkflowStop>();
    type_exists::<WorthQueryWorkflowNextAction>();
}
