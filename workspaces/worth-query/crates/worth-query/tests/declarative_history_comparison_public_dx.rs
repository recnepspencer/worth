use crate::support;

use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn history_transcript_executes_using_only_history_capability_vocabulary() {
    use worth_query::facade::history::{
        at, declare, AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName,
        QuerySchemaView, ScalarAspectType, SchemaFieldView,
    };

    let declaration = declare(|read| {
        read.local_detail(
            "user",
            QuerySchemaView::new(
                "public-history-dx",
                [SchemaFieldView::new(
                    AspectName::new("identity").expect("aspect should build"),
                    FieldName::new("id").expect("field should build"),
                    ScalarAspectType::String,
                )],
                [],
            ),
            |query| {
                query.project(
                    AspectFieldSelector::new("identity", "id").expect("projection should build"),
                )
            },
            |shape| {
                shape.field(
                    AuthoredResultShapeField::new("identity", "id", "id")
                        .expect("result field should build"),
                )
            },
        )
    })
    .expect("history should declare")
    .retained_snapshot();
    let runtime = PublicBridgeRuntimeHarness::new().bridge_backed_runtime();
    let mut workspace = runtime
        .workspace("public-history-dx")
        .expect("workspace should open");
    let context = at(&workspace).expect("the fixture requires an open workspace");
    let outcome = declaration.using(context).run(&mut workspace);

    assert!(outcome.completed().is_some());
    assert!(outcome.stop().is_none());
}

#[test]
fn comparison_transcript_executes_using_only_comparison_capability_vocabulary() {
    use worth_query::facade::comparison::{
        between, declare, AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName,
        QuerySchemaView, ScalarAspectType, SchemaFieldView, WorthQueryComparisonBasisFamily,
        WorthQueryComparisonChange, WorthQuerySessionLabel,
    };

    let declaration = declare(|read| {
        read.local_detail(
            "user",
            QuerySchemaView::new(
                "public-comparison-dx",
                [SchemaFieldView::new(
                    AspectName::new("identity").expect("aspect should build"),
                    FieldName::new("id").expect("field should build"),
                    ScalarAspectType::String,
                )],
                [],
            ),
            |query| {
                query.project(
                    AspectFieldSelector::new("identity", "id").expect("projection should build"),
                )
            },
            |shape| {
                shape.field(
                    AuthoredResultShapeField::new("identity", "id", "id")
                        .expect("result field should build"),
                )
            },
        )
    })
    .expect("comparison should declare")
    .diff();
    let mut left = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime()
        .workspace("public-comparison-left")
        .expect("left workspace should open");
    let mut right = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime()
        .workspace("public-comparison-right")
        .expect("right workspace should open");
    let context = between(
        &left,
        WorthQuerySessionLabel::scoped_strs("public-comparison", ["left"])
            .expect("left branch label should build"),
        &right,
        WorthQuerySessionLabel::scoped_strs("public-comparison", ["right"])
            .expect("right branch label should build"),
    )
    .expect("branch bases should admit");
    let outcome = declaration.using(context).run((&mut left, &mut right));

    let completion = outcome.completed().expect("diff should complete");
    assert_eq!(completion.change(), WorthQueryComparisonChange::Unchanged);
    assert_eq!(
        completion.basis_pair().family(),
        WorthQueryComparisonBasisFamily::BranchToBranch
    );
}
