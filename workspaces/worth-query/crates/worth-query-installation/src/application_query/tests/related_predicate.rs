//! Installed related-record absence/existence predicates compile into graph
//! access contracts, register traversed relations in planning inventory, and
//! contribute distinct graph and planning canonical digests.
use super::{account_parameter, definition, AccountActivity, ActivitySequence};
use crate::application_query::{
    WorthQueryInstalledGraphReadContract, WorthQueryReadGraphPlanningContract,
    WorthQueryReadGraphRelationDirection,
};
use worth_foundational::facade::{CanonicalDigestId, CanonicalDigestWorkBudget};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryOrderingDirection, ApplicationQueryRelatedPolarity,
    ApplicationQueryRelatedPredicate,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, U64ApplicationValueBinding,
};

fn compile_related(
    polarity: Option<ApplicationQueryRelatedPolarity>,
    with_target_literal: bool,
) -> WorthQueryInstalledGraphReadContract {
    let mut builder = definition(ApplicationQueryOrderingDirection::Descending, "sequence")
        .parameter(account_parameter());
    if let Some(polarity) = polarity {
        let mut pred = ApplicationQueryRelatedPredicate::forward(AccountActivity::reference());
        if with_target_literal {
            let ten = ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(10)
                .expect("10 encodes");
            pred = pred.where_equal_value(ActivitySequence::at("ActivitySequence"), ten);
        }
        builder = match polarity {
            ApplicationQueryRelatedPolarity::Absent => builder.where_no_related(pred),
            ApplicationQueryRelatedPolarity::Exists => builder.where_related_exists(pred),
        };
    }
    WorthQueryInstalledGraphReadContract::compile(
        &builder.into_erased(),
        &CanonicalDigestId::new([1; 32]),
        CanonicalDigestWorkBudget::new(8_192, 1024 * 1024).unwrap(),
    )
    .expect("installed related predicate compiles within budget")
}

#[test]
fn related_predicates_compile_into_access_contract_planning_inventory_and_digests() {
    let plain = compile_related(None, false);
    let absent_lit = compile_related(Some(ApplicationQueryRelatedPolarity::Absent), true);
    let exists_lit = compile_related(Some(ApplicationQueryRelatedPolarity::Exists), true);
    let absent_any = compile_related(Some(ApplicationQueryRelatedPolarity::Absent), false);

    assert!(plain.related_predicates().is_empty());
    assert_eq!(absent_lit.related_predicates().len(), 1);
    assert!(absent_lit.has_extended_read_scope());

    // Planning contract and admitted inventory both expose the traversed relation.
    assert_eq!(absent_lit.relation_count(), plain.relation_count() + 1);
    let added = absent_lit
        .relation(plain.relation_count())
        .expect("related predicate relation is exposed in planning contract");
    assert_eq!(added.relation, "AccountActivity");
    assert_eq!(
        added.direction,
        WorthQueryReadGraphRelationDirection::Forward
    );

    let inventory = absent_lit
        .admitted_planning_inventory(&mut |_, _| Ok::<(), std::convert::Infallible>(()))
        .unwrap();
    assert_eq!(inventory.relation_count(), absent_lit.relation_count());
    let relations: Vec<_> = inventory
        .relations_admitted(&mut |_, _| Ok::<(), std::convert::Infallible>(()))
        .unwrap()
        .collect();
    assert_eq!(relations.len(), absent_lit.relation_count());
    assert_eq!(relations.last().unwrap().relation, "AccountActivity");

    // Distinct graph and planning digests across polarity and target predicate.
    for (a, b) in [
        (&plain, &absent_lit),
        (&absent_lit, &exists_lit),
        (&absent_lit, &absent_any),
    ] {
        assert_ne!(a.digest(), b.digest());
        assert_ne!(
            a.canonical_planning_basis().digest(),
            b.canonical_planning_basis().digest()
        );
    }
}
