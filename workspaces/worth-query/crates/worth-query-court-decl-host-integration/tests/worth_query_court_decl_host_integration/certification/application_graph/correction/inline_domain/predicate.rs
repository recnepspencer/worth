//! A corrupt opaque payload cannot turn codec failure into plausible suppression.
use super::super::model::{composite_definition_contract, composite_definition_runtime::Predicate};
use super::*;
use worth_foundational::facade::{
    validate_aspect_value, ContractValidationInput, FieldKey, InlineBytes, StructAspectValue,
};
use worth_query_host::facade::domain::{self, WorthQueryHostConditionalPredicateProvider};

#[test]
fn inline_domain_predicate_refuses_corrupt_and_unobserved_values_but_preserves_absence() {
    let application = host::install();
    let contract = application
        .installed_schema()
        .native_contracts()
        .aspect(
            "Number",
            super::super::model::schema::NumberFacts::reference().name(),
        )
        .unwrap()
        .contract();
    // Independent descriptive provider fixture, validated by the installed contract.
    let fields = StructAspectValue::new(
        [
            (
                "Key",
                AspectValue::String(worth_foundational::InternedString::Raw(
                    "numeric".to_owned(),
                )),
            ),
            (
                "Primary",
                AspectValue::InlineBytes(InlineBytes::try_new(FIVE.to_vec()).unwrap()),
            ),
            ("Secondary", AspectValue::UInt64(7)),
            ("ReadEnabled", AspectValue::Bool(true)),
            ("Ceiling", AspectValue::UInt64(100)),
            ("Calculated", AspectValue::UInt64(0)),
        ]
        .map(|(name, value)| (FieldKey::new(name).unwrap(), value)),
    )
    .unwrap();
    let mask = composite_definition_contract::mask();
    let basis = domain::WorthQueryConditionalObservationTruthBasis::from_runtime_truth(
        "pure-provider-branch",
        "pure-provider-snapshot",
    );
    for version in [1, 2] {
        let raw = AspectValue::InlineBytes(
            InlineBytes::try_new(vec![version, 5, 0, 0, 0, 0, 0, 0, 0]).unwrap(),
        );
        let changed = StructAspectValue::new(fields.fields().map(|(key, value)| {
            (
                key.clone(),
                if key.as_str() == "Primary" {
                    raw.clone()
                } else {
                    value.clone()
                },
            )
        }))
        .unwrap();
        // Native validates the opaque family. Only its domain owner can reject
        // the corrupted version, before a provider makes a truth decision.
        let artifact =
            validate_aspect_value(contract, ContractValidationInput::Struct(changed)).unwrap();
        let projected =
            domain::WorthQueryConditionalProjectedValue::from_runtime_projection(&artifact, &mask);
        let dependencies = [
            domain::WorthQueryConditionalDependencyObservation::from_runtime_observation(
                0,
                domain::WorthQueryConditionalObservedValue::Absent,
                domain::WorthQueryConditionalObservedValue::Present(projected),
            ),
        ];
        let view = domain::WorthQueryConditionalObservationView::from_runtime_observations(
            basis,
            &dependencies,
        );
        let result = Predicate(5).evaluate(view);
        if version == 1 {
            assert_eq!(
                result.unwrap(),
                domain::WorthQueryHostPredicateDecision::Satisfied
            );
        } else {
            let denial = result.unwrap_err();
            assert_eq!(
                denial.kind(),
                domain::WorthQueryHostPredicateFailureKind::ObservationUnsupported
            );
            assert!(denial
                .detail()
                .contains("numeric-correction.inline-counter.v1"));
            assert!(denial.detail().contains("UnsupportedVersion"));
        }
    }
    let absent = [
        domain::WorthQueryConditionalDependencyObservation::from_runtime_observation(
            0,
            domain::WorthQueryConditionalObservedValue::Absent,
            domain::WorthQueryConditionalObservedValue::Absent,
        ),
    ];
    assert_eq!(
        Predicate(5)
            .evaluate(
                domain::WorthQueryConditionalObservationView::from_runtime_observations(
                    basis, &absent
                )
            )
            .unwrap(),
        domain::WorthQueryHostPredicateDecision::Unsatisfied
    );
    assert_eq!(
        Predicate(5)
            .evaluate(
                domain::WorthQueryConditionalObservationView::from_runtime_observations(basis, &[])
            )
            .unwrap_err()
            .kind(),
        domain::WorthQueryHostPredicateFailureKind::ObservationUnsupported
    );
}
