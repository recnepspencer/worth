use std::path::PathBuf;

use worth_ui_dsl::{
    WorthUiAuthoredSourceInput, WorthUiDslCompiler, WorthUiSealedExpression,
    WorthUiSealedSemanticPackage,
};
use worth_ui_query_binding::WorthUiQueryBindingPlan;

use super::{
    UiExpressionCatalog, UiExpressionCatalogPreparationDenial as Denial,
    UiResolvedExpressionOperand, WorthUiAuthoredExpressionMaterial,
};
use crate::capability::UiIntentPayloadFieldKind;
use crate::declaration::{UiIntentApplicationFact, UiIntentApplicationFactPlan};

const READY: &str = "app.ready";
const COUNT: &str = "app.count";

const PROJECTIONS: &str =
    "query_scalar pulse.label { view pulse.label field label require text }\n";

const READS_READY: &str =
    "condition ex.ready { operand r application-boolean app.ready; when (r) }";

fn package(declarations: &str) -> WorthUiSealedSemanticPackage {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("main.wui", format!("{PROJECTIONS}{declarations}")),
    )
    .expect("installation fixture source seals")
}

fn expression(package: &WorthUiSealedSemanticPackage, identity: &str) -> WorthUiSealedExpression {
    package
        .expression(identity)
        .expect("the fixture declares this expression")
        .clone()
}

fn material(expressions: Vec<WorthUiSealedExpression>) -> WorthUiAuthoredExpressionMaterial {
    WorthUiAuthoredExpressionMaterial {
        expressions: expressions.into_boxed_slice(),
    }
}

fn facts() -> UiIntentApplicationFactPlan {
    let mut plan = UiIntentApplicationFactPlan::default();
    plan.register_boolean(UiIntentApplicationFact::boolean(READY).unwrap(), false)
        .unwrap();
    plan.register_unsigned64(UiIntentApplicationFact::unsigned64(COUNT).unwrap(), 3)
        .unwrap();
    plan
}

fn prepare_material(
    material: &WorthUiAuthoredExpressionMaterial,
    facts: &UiIntentApplicationFactPlan,
) -> Result<UiExpressionCatalog, Denial> {
    UiExpressionCatalog::prepare(material, &[], &WorthUiQueryBindingPlan::default(), facts)
}

fn prepare(
    package: &WorthUiSealedSemanticPackage,
    facts: &UiIntentApplicationFactPlan,
) -> Result<UiExpressionCatalog, Denial> {
    prepare_material(
        &WorthUiAuthoredExpressionMaterial::from_package(package),
        facts,
    )
}

#[test]
fn an_unregistered_application_fact_is_denied() {
    let denial = prepare(
        &package(READS_READY),
        &UiIntentApplicationFactPlan::default(),
    )
    .expect_err("the fact is not registered");

    assert_eq!(
        denial,
        Denial::UnknownApplicationFact {
            expression: "ex.ready".into(),
            operand: "r".into(),
            fact: READY.into(),
        }
    );
}

#[test]
fn an_application_fact_of_another_kind_is_denied() {
    let package =
        package("condition ex.wrong { operand c application-boolean app.count; when (c) }");

    let denial = prepare(&package, &facts()).expect_err("`app.count` is unsigned64");

    assert_eq!(
        denial,
        Denial::ApplicationFactKindMismatch {
            expression: "ex.wrong".into(),
            operand: "c".into(),
            fact: COUNT.into(),
            expected: UiIntentPayloadFieldKind::Boolean,
            observed: UiIntentPayloadFieldKind::Unsigned64,
        }
    );
}

#[test]
fn a_query_scalar_with_no_scalar_requirement_is_denied() {
    let package =
        package("condition ex.label { operand l query-scalar pulse.label; when (l == \"x\") }");

    let denial = prepare(&package, &facts()).expect_err("no projection requirements are given");

    assert_eq!(
        denial,
        Denial::UnknownQueryScalar {
            expression: "ex.label".into(),
            operand: "l".into(),
            projection: "pulse.label".into(),
        }
    );
}

#[test]
fn an_expression_operand_missing_from_the_package_is_denied() {
    let package = package(&format!(
        "{READS_READY}\ncondition ex.reader {{ operand up condition ex.ready; when (up) }}"
    ));

    let denial = prepare_material(&material(vec![expression(&package, "ex.reader")]), &facts())
        .expect_err("`ex.ready` is not installed");

    assert_eq!(
        denial,
        Denial::UnknownExpressionOperand {
            expression: "ex.reader".into(),
            operand: "up".into(),
            identity: "ex.ready".into(),
        }
    );
}

#[test]
fn expressions_that_read_each_other_are_a_cycle() {
    // The DSL refuses cycles, so the cycle is built from two packages that
    // each declare one half of it.
    let reads_b = package(
        "condition ex.b { operand r application-boolean app.ready; when (r) }\n\
         condition ex.a { operand up condition ex.b; when (up) }",
    );
    let reads_a = package(
        "condition ex.a { operand r application-boolean app.ready; when (r) }\n\
         condition ex.b { operand up condition ex.a; when (up) }",
    );

    let denial = prepare_material(
        &material(vec![
            expression(&reads_b, "ex.a"),
            expression(&reads_a, "ex.b"),
        ]),
        &facts(),
    )
    .expect_err("`ex.a` and `ex.b` read each other");

    assert!(
        matches!(&denial, Denial::ExpressionCycle { expression } if &**expression == "ex.a" || &**expression == "ex.b"),
        "{denial:?}"
    );
}

#[test]
fn more_expressions_than_one_catalog_installs_are_denied() {
    let package = package(READS_READY);
    let one = expression(&package, "ex.ready");

    let denial = prepare_material(&material(vec![one; 65_537]), &facts())
        .expect_err("65,537 exceeds the limit");

    assert_eq!(
        denial,
        Denial::CapacityExceeded {
            observed: 65_537,
            maximum: 65_536,
        }
    );
}

#[test]
fn slots_are_topological_ranks_not_identity_order() {
    let package = package(
        "condition ex.a_top { operand mid condition ex.m_mid; when (mid) }\n\
         condition ex.m_mid { operand base condition ex.z_base; when (base) }\n\
         condition ex.z_base { operand r application-boolean app.ready; when (r) }",
    );

    let catalog = prepare(&package, &facts()).expect("the chain installs");

    let slots: Vec<_> = ["ex.z_base", "ex.m_mid", "ex.a_top"]
        .into_iter()
        .map(|identity| catalog.slot_of(identity).unwrap().index())
        .collect();
    assert_eq!(slots, [0, 1, 2], "every upstream ranks before its readers");
    assert_eq!(catalog.slot_count().slots().count(), 3);
    for (rank, slot) in catalog.slot_count().slots().enumerate() {
        let installed = catalog.expression(slot).unwrap();
        for operand in installed.operands() {
            if let UiResolvedExpressionOperand::Expression {
                slot: upstream,
                identity,
            } = operand.source()
            {
                assert_eq!(
                    catalog.expression(*upstream).unwrap().identity(),
                    &**identity
                );
                assert!(upstream.index() < rank);
            }
        }
    }
}

const DIAMOND: &str = "condition ex.z_source { operand r application-boolean app.ready; when (r) }\n\
    condition ex.b_left { operand s condition ex.z_source; when (s) }\n\
    condition ex.c_right { operand s condition ex.z_source; when (s) }\n\
    condition ex.d_join { operand l condition ex.b_left; operand r condition ex.c_right; when (l && r) }";

#[test]
fn a_diamond_indexes_each_reader_under_each_of_its_operands() {
    let plan = facts();
    let catalog = prepare(&package(DIAMOND), &plan).expect("the diamond installs");
    let slot = |identity: &str| catalog.slot_of(identity).unwrap();
    let source = slot("ex.z_source");
    let left = slot("ex.b_left");
    let right = slot("ex.c_right");
    let join = slot("ex.d_join");

    assert_eq!(
        (source.index(), left.index(), right.index(), join.index()),
        (0, 1, 2, 3)
    );
    let index = catalog.dependencies();
    assert_eq!(index.dependents_of(source), [left, right]);
    assert_eq!(index.dependents_of(left), [join]);
    assert_eq!(index.dependents_of(right), [join]);
    assert!(index.dependents_of(join).is_empty());
    let ready = plan.get(READY).unwrap().slot();
    let count = plan.get(COUNT).unwrap().slot();
    assert_eq!(index.readers_of_application(ready), [source]);
    assert!(index.readers_of_application(count).is_empty());
    assert_eq!(index.projection_slots().count(), 0);
}

#[test]
fn an_installed_expression_keeps_its_program_identity_operand_order_and_span() {
    let package = package(
        "condition ex.pair { operand zed application-boolean app.ready; operand alpha application-unsigned64 app.count; when (zed && exact_cast<Int64>(alpha) > 1) }",
    );
    let sealed = package.expression("ex.pair").unwrap();

    let catalog = prepare(&package, &facts()).expect("the pair installs");
    let installed = catalog
        .expression(catalog.slot_of("ex.pair").unwrap())
        .unwrap();

    let names: Vec<_> = installed
        .operands()
        .iter()
        .map(|operand| operand.name())
        .collect();
    assert_eq!(names, ["alpha", "zed"], "operands are in name order");
    assert_eq!(installed.program_identity(), sealed.program_identity());
    assert_eq!(
        installed.body_span().map(|span| &**span),
        sealed.body_span()
    );
    assert!(installed.body_span().is_some());
}
