//! A where filter lowers to exact field reads and joins graph identity.
use super::{account_parameter, definition, AccountId};
use worth_foundational::expression_api::expressions;
use worth_foundational::facade::{CanonicalDigestId, CanonicalDigestWorkBudget};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryOrderingDirection, ApplicationQueryWhereFilterOperands,
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
