//! Admission layout validation, canonical distinctness, and work/memory
//! metering for optional query parameters (`G7`).
use super::*;
use crate::facade::application_query::{
    admit_application_query_parameters_admitted, readmit_recorded_application_query_parameters,
    WorthQueryApplicationQueryParameterDenialKind as DenialKind,
    WorthQueryParameterPreparationStop,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryWhereFilterOperands;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

struct OptionalSearchParam;
struct OptionalSearchSlot;
worth_query_portable_type!(OptionalSearchSlot => "worth.query.test.admission.optional-search-slot.v1");
worth_query_application_query!(
    pub(super) OptionalSearchQuery for PlanningTestSchema,
    identity "worth.query.test.admission.optional-search.v1",
    parameters ActivityParametersBinding,
    result ActivityResultBinding,
    scope Account => "Account",
    name "account_optional_search"
);

fn optional_text_param() -> ApplicationQueryParameterRef<
    OptionalSearchQuery,
    OptionalSearchParam,
    StringApplicationValueBinding,
    true,
> {
    ApplicationQueryParameterRef::optional("text")
}

pub(super) fn definition() -> ApplicationQueryDefinitionBuilder<
    PlanningTestSchema,
    OptionalSearchQuery,
    ActivityParameters,
    ActivityResult,
    Account,
> {
    let text = optional_text_param();
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("id", AccountId::at("AccountId"))
        .and_then(|operands| operands.parameter("text", text))
        .and_then(|operands| operands.parse("!is_some(text) || id == id"))
        .expect("optional parameter filter admits");
    let shape = ApplicationQueryResultShapeBuilder::<
        PlanningTestSchema,
        OptionalSearchQuery,
        Account,
        ActivityResult,
        ActivityResultBinding,
    >::new(Account::reference())
    .field(ApplicationQueryResultFieldRef::<
        OptionalSearchQuery,
        OptionalSearchSlot,
        PlanningTestSchema,
        AccountId,
    >::new("account_id", AccountId::at("AccountId")))
    .build();
    ApplicationQueryDefinitionBuilder::declare(OptionalSearchQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(text)
        .where_filter(filter)
}

#[test]
fn optional_parameter_admission_validates_layout_distinguishes_none_from_empty_and_meters_work() {
    let optional_query = installed_optional_search_query();
    let required_query = installed_query();

    let absent = admit_application_query_parameters(
        &optional_query,
        ApplicationQueryParameterSet::new()
            .bind_optional(optional_text_param(), None)
            .unwrap(),
    )
    .expect("None admits for an optional parameter");
    let empty = admit_application_query_parameters(
        &optional_query,
        ApplicationQueryParameterSet::new()
            .bind_optional(optional_text_param(), Some(String::new()))
            .unwrap(),
    )
    .expect("Some(empty) admits for an optional parameter");
    assert_ne!(
        absent.identity(),
        empty.identity(),
        "None and Some(\"\") must never share canonical parameter identity"
    );

    // Wrong type for optional parameter refuses ParameterTypeMismatch.
    let wrong_type = readmit_recorded_application_query_parameters(
        &optional_query,
        vec![("text".to_owned(), AspectValue::Int64(42))],
    )
    .expect_err("wrong scalar type refuses for optional parameter");
    assert_eq!(wrong_type.kind(), DenialKind::ParameterTypeMismatch);

    // Missing optional parameter refuses ParameterSetMismatch.
    let missing =
        admit_application_query_parameters(&optional_query, ApplicationQueryParameterSet::new())
            .expect_err("omitting an optional parameter refuses");
    assert_eq!(missing.kind(), DenialKind::ParameterSetMismatch);

    // Null for a required parameter refuses ParameterTypeMismatch.
    let null_for_required = readmit_recorded_application_query_parameters(
        &required_query,
        vec![("account".to_owned(), AspectValue::Null)],
    )
    .expect_err("Null refuses for a required parameter");
    assert_eq!(null_for_required.kind(), DenialKind::ParameterTypeMismatch);

    // Work & memory metering: Some(large_string) charges strictly more than None.
    let mut none_work = 0usize;
    let mut none_bytes = 0usize;
    admit_application_query_parameters_admitted(
        &optional_query,
        ApplicationQueryParameterSet::new()
            .bind_optional(optional_text_param(), None)
            .unwrap(),
        |work, bytes| {
            none_work += work;
            none_bytes += bytes;
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    let mut large_work = 0usize;
    let mut large_bytes = 0usize;
    let payload = "x".repeat(2048);
    admit_application_query_parameters_admitted(
        &optional_query,
        ApplicationQueryParameterSet::new()
            .bind_optional(optional_text_param(), Some(payload))
            .unwrap(),
        |work, bytes| {
            large_work += work;
            large_bytes += bytes;
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert!(
        large_work > none_work + 2048,
        "Some(2 KiB) must charge work beyond None: {none_work} vs {large_work}"
    );
    assert!(
        large_bytes > none_bytes + 2048,
        "Some(2 KiB) must charge memory beyond None: {none_bytes} vs {large_bytes}"
    );

    // Budget refusal stops before cloning a large optional string.
    let refused = admit_application_query_parameters_admitted(
        &optional_query,
        ApplicationQueryParameterSet::new()
            .bind_optional(optional_text_param(), Some("z".repeat(2048)))
            .unwrap(),
        |work, _| {
            if work > 100 {
                Err("budget")
            } else {
                Ok(())
            }
        },
    );
    assert!(matches!(
        refused,
        Err(WorthQueryParameterPreparationStop::Admission("budget"))
    ));
}
