use super::product_support as fixture;

#[test]
fn equivalent_declarations_converge() {
    use worth_query::facade::read::{
        declare, AspectFieldSelector, AuthoredResultShapeField, CollectionQueryBuilder,
        CollectionResultShapeBuilder, QueryScopeDescriptor, QueryTemplateDescriptor, RootEntityKey,
        TemplateBindingSet, TemplateParameterSlot,
    };
    let identity = || AspectFieldSelector::new("identity", "id").unwrap();
    let display = || AspectFieldSelector::new("profile", "display_name").unwrap();
    let identity_field = || AuthoredResultShapeField::new("identity", "id", "id").unwrap();
    let display_field =
        || AuthoredResultShapeField::new("profile", "display_name", "display_name").unwrap();
    let schema = || {
        use worth_query::facade::read::{
            AspectName, FieldName, QuerySchemaView, ScalarAspectType, SchemaFieldView,
        };
        QuerySchemaView::new(
            "hostile-convergence",
            [
                SchemaFieldView::new(
                    AspectName::new("identity").unwrap(),
                    FieldName::new("id").unwrap(),
                    ScalarAspectType::String,
                ),
                SchemaFieldView::new(
                    AspectName::new("profile").unwrap(),
                    FieldName::new("display_name").unwrap(),
                    ScalarAspectType::String,
                ),
            ],
            [],
        )
    };
    let direct = declare(|read| {
        read.local_collection(
            "Task",
            schema(),
            |query| query.project(identity()).project(display()),
            |shape| shape.field(identity_field()).field(display_field()),
        )
    })
    .unwrap();
    let scoped_query = CollectionQueryBuilder::new(RootEntityKey::new("Task").unwrap())
        .project(identity())
        .build()
        .unwrap();
    let scoped_shape = CollectionResultShapeBuilder::new()
        .field(identity_field())
        .field(display_field())
        .build()
        .unwrap();
    let scoped = declare(|read| {
        read.local_collection_scoped(
            scoped_query.clone(),
            scoped_shape.clone(),
            schema(),
            [QueryScopeDescriptor::projection("display", [display()])],
        )
    })
    .unwrap();
    let slot = TemplateParameterSlot::projection("display");
    let template =
        QueryTemplateDescriptor::collection(scoped_query, scoped_shape).with_slot(slot.clone());
    let templated = declare(|read| {
        read.local_collection_template(
            template,
            TemplateBindingSet::new().bind_projection(&slot, display()),
            schema(),
        )
    })
    .unwrap();

    for equivalent in [&scoped, &templated] {
        assert_eq!(
            direct.identity().canonical_query_digest(),
            equivalent.identity().canonical_query_digest()
        );
        assert_eq!(
            direct.identity().canonical_result_shape_digest(),
            equivalent.identity().canonical_result_shape_digest()
        );
    }
}

#[test]
fn cross_basis_denies_before_execution() {
    use worth_query::facade::history::{at, declare, WorthQueryHistoricalStopSource};
    let left = fixture::workspace("hostile-cross-basis-left");
    let mut right = fixture::workspace("hostile-cross-basis-right");
    let outcome = declare(fixture::identity_detail)
        .unwrap()
        .retained_snapshot()
        .using(at(&left).expect("the fixture requires an open workspace"))
        .run(&mut right);
    let stop = outcome.stop().expect("foreign basis must stop");
    assert_eq!(stop.source(), WorthQueryHistoricalStopSource::StaleContext);
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}

#[test]
fn stale_context_denies_before_execution() {
    use worth_query::facade::history::{at, declare, WorthQueryHistoricalStopSource};
    let mut workspace = fixture::workspace("hostile-stale-context");
    let context = at(&workspace).expect("the fixture requires an open workspace");
    fixture::write_task(&mut workspace, "basis-advance");
    let outcome = declare(fixture::identity_detail)
        .unwrap()
        .retained_snapshot()
        .using(context)
        .run(&mut workspace);
    let stop = outcome.stop().expect("stale basis must stop");
    assert_eq!(stop.source(), WorthQueryHistoricalStopSource::StaleContext);
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}

#[test]
fn one_shot_and_live_results_match() {
    use worth_query::facade::live::{
        current as live_current, declare as declare_live, WorthQueryLiveOpenOutcome,
    };
    use worth_query::facade::read::{current, declare};
    let mut workspace = fixture::workspace("hostile-one-shot-live");
    fixture::write_task(&mut workspace, "same-row");
    let declaration = declare(fixture::identity_collection).unwrap();
    let canonical_result_shape_digest = declaration
        .identity()
        .canonical_result_shape_digest()
        .to_string();
    let one_shot = declaration
        .using(current())
        .run(&mut workspace)
        .into_result()
        .unwrap();
    let one_shot_context = one_shot.context_receipt().clone();
    let one_shot = one_shot.into_result();
    let opened = declare_live("hostile.parity", fixture::identity_collection)
        .unwrap()
        .using(live_current())
        .open(&mut workspace);
    let opened = match opened {
        WorthQueryLiveOpenOutcome::Opened(value) => value,
        WorthQueryLiveOpenOutcome::Stopped(stop) => panic!("live stopped: {:?}", stop.source()),
    };
    assert_eq!(&one_shot_context, opened.context_receipt());
    let handle = opened.into_handle();
    let live = handle
        .read(&mut workspace)
        .expect("live read should succeed");
    assert_eq!(one_shot.rows(), live.rows());
    assert_eq!(
        one_shot.receipt().snapshot_identity(),
        live.receipt().snapshot_identity()
    );
    assert_eq!(
        one_shot.receipt().snapshot_evidence_identity(),
        *live.receipt().snapshot_evidence_identity()
    );
    assert_eq!(
        canonical_result_shape_digest,
        live.receipt().view_shape_digest()
    );
    assert!(matches!(
        handle.close(&mut workspace),
        worth_query::facade::live::WorthQueryManagedLiveCloseOutcome::Closed(_)
    ));
}

#[test]
fn historical_ambiguity_remains_advisory() {
    use worth_query::facade::comparison::{
        between, declare, WorthQueryComparisonCorrespondencePosture, WorthQuerySessionLabel,
    };
    let mut left = fixture::workspace("hostile-ambiguity-left");
    let mut right = fixture::workspace("hostile-ambiguity-right");
    fixture::write_task(&mut left, "subject");
    fixture::write_task(&mut right, "candidate-one");
    fixture::write_task(&mut right, "candidate-two");
    let context = between(
        &left,
        WorthQuerySessionLabel::scoped_strs("hostile", ["left"]).unwrap(),
        &right,
        WorthQuerySessionLabel::scoped_strs("hostile", ["right"]).unwrap(),
    )
    .unwrap();
    let outcome = declare(fixture::identity_collection)
        .unwrap()
        .correspondence(4)
        .using(context)
        .run((&mut left, &mut right));
    assert!(outcome.completed().is_none());
    assert!(outcome.stop().is_none());
    let evidence = outcome
        .correspondence()
        .expect("ambiguity must remain evidence");
    assert_eq!(
        evidence.posture(),
        WorthQueryComparisonCorrespondencePosture::Advisory
    );
    assert_eq!(
        evidence
            .correspondence()
            .outcome()
            .as_advisory_structural_ambiguous()
            .unwrap()
            .candidate_set()
            .len(),
        2
    );
}

#[test]
fn preview_workflow_cross_session_denies() {
    use worth_query::facade::workflow::{
        declare, preview, WorthQuerySessionLabel, WorthQueryWorkflowStopSource,
    };
    let mut workspace = fixture::workspace("hostile-cross-session");
    let declared = WorthQuerySessionLabel::scoped_strs("hostile", ["declared"]).unwrap();
    let foreign = WorthQuerySessionLabel::scoped_strs("hostile", ["foreign"]).unwrap();
    let context = preview(&workspace, foreign).expect("foreign context itself should admit");
    let outcome = declare(declared, fixture::mutation("cross-session"))
        .using(context)
        .run(&mut workspace);
    let stop = outcome.stop().expect("cross-session workflow must stop");
    assert_eq!(stop.source(), WorthQueryWorkflowStopSource::CrossSession);
    assert_eq!(stop.counters().lower_runtime_execution_attempt_count(), 0);
}

#[test]
fn diagnostic_policy_preserves_operational_truth() {
    use worth_query::facade::inspection::{declare, inspection_basis};
    use worth_query::facade::read::{current, declare as declare_read};
    let mut workspace = fixture::workspace("hostile-diagnostic-policy");
    let completion = declare_read(fixture::identity_detail)
        .unwrap()
        .using(current())
        .run(&mut workspace)
        .into_result()
        .unwrap();
    let basis = fixture::inspection_basis("diagnostic-policy");
    let operational = declare(&completion)
        .using(inspection_basis(basis.clone()))
        .run(&workspace);
    let rich = declare(&completion)
        .with_rich_inspection()
        .using(inspection_basis(basis))
        .run(&workspace);
    let operational = operational
        .settled()
        .expect("operational inspection should settle");
    let rich = rich.settled().expect("rich inspection should settle");
    assert_eq!(operational.receipt(), rich.receipt());
    assert!(operational.materialization().is_none());
    assert!(rich.materialization().is_some());
    assert_eq!(operational.counters().materialization_attempt_count(), 0);
    assert_eq!(rich.counters().materialization_attempt_count(), 1);
}
