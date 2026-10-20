//! Conditional binding identity cannot collapse into operation-only meaning.
use super::*;
use crate::application_program::{
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramOutputs, ApplicationRuleLeaf,
    ApplicationSemanticDiff, ApplicationSemanticFamily,
};
use crate::application_schema::ApplicationOperationMarkerIdentity;

// Deliberately aliases the operation's portable name: the native conditional
// binding tag, not a conveniently different string, must distinguish meaning.
crate::worth_query_mutation_binding!(@binding
    ConditionalBinding for RenameInput, schema MutationSchema,
    identity "RenameOperation", input RenameInputBinding, operation RenameOperation,
    result RenameResultBinding, idempotency u64, identity "conditional-key.v1",
    decision RenameDecision, denial RenameDenialBinding,
    handler identity "conditional-handler.v1", program required,
    workflow_authority , workflow_requirements , outputs RenameOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Account,
        principal_identity u64, identity_binding crate::application_schema::U64ApplicationValueBinding,
    scope_binding crate::application_operation::ApplicationMutationFieldScope<MutationSchema,
        Account, AccountIdentity, AccountIdField, u64,
        crate::application_schema::ReadOnly, crate::application_schema::NoApplicationUnit>,
    scope_field_type Account, AccountIdentity, AccountIdField, u64,
        crate::application_schema::ReadOnly, crate::application_schema::NoApplicationUnit,
    scope_field AccountIdField::reference(),
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 0,
    resources retained_representation_bytes 128
);

struct Feature;
impl ApplicationFeature<MutationSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "conditional-feature.v1";
}
struct Legacy;
struct Bound;
macro_rules! program {
    ($program:ty, $feature:expr) => {
        impl ApplicationProgramDefinition<MutationSchema> for $program {
            type Contributions = ();
            type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
            type Rules = ApplicationRuleLeaf;
            const IDENTITY: ApplicationProgramIdentity =
                ApplicationProgramIdentity::new("conditional-program.v1");
            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                vec![$feature]
            }
        }
    };
}
program!(
    Legacy,
    ApplicationFeatureSpec::root::<MutationSchema, Feature>()
        .conditional_operation::<RenameOperation>()
        .finish()
);
program!(
    Bound,
    ApplicationFeatureSpec::root::<MutationSchema, Feature>()
        .conditional_mutation::<ConditionalBinding>()
        .finish()
);

#[test]
fn conditional_mutation_retains_exact_membership_without_ordinary_call_eligibility() {
    let program = ApplicationProgramAuthoring::<MutationSchema, Bound>::begin()
        .validated_program()
        .unwrap();
    let action = &program.actions()[0];
    assert!(action.conditional_only());
    assert_eq!(
        action.mutation_binding_type(),
        Some(TypeId::of::<ConditionalBinding>())
    );
    assert_eq!(action.ordinary_mutation_binding_type(), None);
    assert_eq!(action.operation_type(), TypeId::of::<RenameOperation>());
    assert_eq!(action.binding(), ConditionalBinding::IDENTITY);
}

#[test]
fn conditional_mutation_changes_canonical_and_evolution_meaning_even_with_aliased_names() {
    let legacy = ApplicationProgramAuthoring::<MutationSchema, Legacy>::begin()
        .validated_program()
        .unwrap();
    let bound = ApplicationProgramAuthoring::<MutationSchema, Bound>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(ConditionalBinding::IDENTITY, RenameOperation::IDENTIFIER);
    assert_ne!(legacy.revision(), bound.revision());
    let diff = ApplicationSemanticDiff::compare(
        legacy.semantic_description(),
        bound.semantic_description(),
        1024,
    )
    .unwrap();
    assert!(diff
        .changes()
        .iter()
        .any(|change| change.family() == ApplicationSemanticFamily::Operations));
    assert!(!legacy
        .normalized_manifest()
        .records()
        .iter()
        .any(|record| record.contains("conditional-mutation-binding")));
    assert!(bound
        .normalized_manifest()
        .records()
        .iter()
        .any(|record| record.contains("conditional-mutation-binding")));
}
