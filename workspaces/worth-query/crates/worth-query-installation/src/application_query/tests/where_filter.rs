//! A where filter lowers to exact field reads and joins graph identity.
use super::{account_parameter, definition, AccountId};
use worth_foundational::expression_api::expressions;
use worth_foundational::facade::{CanonicalDigestId, CanonicalDigestWorkBudget};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryOrderingDirection, ApplicationQueryParameterRef,
    ApplicationQueryWhereFilterOperands,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, U64ApplicationValueBinding,
};

fn compile(filtered: bool) -> super::super::WorthQueryInstalledGraphReadContract {
    let mut builder = definition(ApplicationQueryOrderingDirection::Descending, "sequence")
        .parameter(account_parameter());
    if filtered {
        let filter = ApplicationQueryWhereFilterOperands::new()
            .field("id", AccountId::at("AccountId"))
            .unwrap()
            .parameter("account", account_parameter())
            .unwrap()
            .admit(&expressions().parse("id >= account").unwrap())
            .unwrap();
        builder = builder.where_filter(filter);
    }
    super::super::WorthQueryInstalledGraphReadContract::compile(
        &builder.into_erased(),
        &CanonicalDigestId::new([1; 32]),
        CanonicalDigestWorkBudget::new(4_096, 1024 * 1024).unwrap(),
    )
    .expect("the test graph fits its canonical budget")
}

#[test]
fn the_installed_filter_lowers_exact_field_reads_and_changes_graph_identity() {
    let unfiltered = compile(false);
    let filtered = compile(true);
    assert!(unfiltered.where_filter().is_none());
    let filter = filtered.where_filter().expect("the filter is installed");
    let field = AccountId::at("AccountId");
    assert_eq!(
        filter
            .fields()
            .iter()
            .map(|field| (field.operand(), field.field()))
            .collect::<Vec<_>>(),
        [("id", (field.entity(), field.aspect(), field.field()))]
    );
    assert_eq!(filter.parameters()[0].parameter(), "account");
    assert!(
        filtered.has_extended_read_scope(),
        "filter-only reads must overlap conservatively with writers"
    );
    assert_ne!(filtered.digest(), unfiltered.digest());
    assert_ne!(
        filtered.canonical_planning_basis().digest(),
        unfiltered.canonical_planning_basis().digest()
    );
}

#[test]
fn optional_filter_parameters_literals_and_value_predicates_join_graph_and_planning_identity() {
    struct OptionalAccount;
    let budget = CanonicalDigestWorkBudget::new(4_096, 1024 * 1024).unwrap();
    let schema_digest = CanonicalDigestId::new([1; 32]);
    let opt_param = ApplicationQueryParameterRef::<
        super::ActivityQuery,
        OptionalAccount,
        U64ApplicationValueBinding,
        true,
    >::optional("account");
    let lit_seven =
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(7).unwrap();
    let lit_eight =
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(8).unwrap();

    let opt_filter = ApplicationQueryWhereFilterOperands::new()
        .field("id", AccountId::at("AccountId"))
        .unwrap()
        .parameter("account", opt_param)
        .unwrap()
        .literal("floor", lit_seven)
        .unwrap()
        .parse("id >= floor && (!is_some(account) || id == unwrap(account))")
        .unwrap();
    let compiled_a = super::super::WorthQueryInstalledGraphReadContract::compile(
        &definition(ApplicationQueryOrderingDirection::Descending, "sequence")
            .parameter(opt_param)
            .where_filter(opt_filter)
            .into_erased(),
        &schema_digest,
        budget,
    )
    .unwrap();
    let opt_filter_b = ApplicationQueryWhereFilterOperands::new()
        .field("id", AccountId::at("AccountId"))
        .unwrap()
        .parameter("account", opt_param)
        .unwrap()
        .literal("floor", lit_eight)
        .unwrap()
        .parse("id >= floor && (!is_some(account) || id == unwrap(account))")
        .unwrap();
    let compiled_b = super::super::WorthQueryInstalledGraphReadContract::compile(
        &definition(ApplicationQueryOrderingDirection::Descending, "sequence")
            .parameter(opt_param)
            .where_filter(opt_filter_b)
            .into_erased(),
        &schema_digest,
        budget,
    )
    .unwrap();
    let filter_a = compiled_a.where_filter().unwrap();
    assert!(filter_a.parameters()[0].is_optional());
    assert_eq!(filter_a.literals().len(), 1);
    assert_eq!(filter_a.literals()[0].operand(), "floor");
    assert_ne!(compiled_a.digest(), compiled_b.digest());
    assert_ne!(
        compiled_a.canonical_planning_basis().digest(),
        compiled_b.canonical_planning_basis().digest()
    );

    let pred_a = super::super::WorthQueryInstalledGraphReadContract::compile(
        &definition(ApplicationQueryOrderingDirection::Descending, "sequence")
            .where_equal_value(
                AccountId::at("AccountId"),
                ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(7).unwrap(),
            )
            .into_erased(),
        &schema_digest,
        budget,
    )
    .unwrap();
    let pred_b = super::super::WorthQueryInstalledGraphReadContract::compile(
        &definition(ApplicationQueryOrderingDirection::Descending, "sequence")
            .where_equal_value(
                AccountId::at("AccountId"),
                ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(8).unwrap(),
            )
            .into_erased(),
        &schema_digest,
        budget,
    )
    .unwrap();
    let literal_pred = pred_a
        .predicates()
        .iter()
        .find(|predicate| predicate.literal_value().is_some())
        .expect("where_equal_value installs a literal predicate");
    assert!(literal_pred.parameter().is_none());
    assert_ne!(pred_a.digest(), pred_b.digest());
    assert_ne!(
        pred_a.canonical_planning_basis().digest(),
        pred_b.canonical_planning_basis().digest()
    );
}
