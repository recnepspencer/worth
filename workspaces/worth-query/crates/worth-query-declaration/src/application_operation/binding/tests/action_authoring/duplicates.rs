//! Portable resolution identity and native type identity share one validator.
use super::*;

// This declaration shares valid domain contracts, not another intent impl.
// Only its Rust binding type differs; the portable resolution key collides.
struct SameIdentityBinding;

impl ApplicationMutationBinding<MutationSchema> for SameIdentityBinding {
    fn workflow_execution_requirements(
    ) -> crate::application_operation::ApplicationWorkflowExecutionRequirements {
        RenameBinding::workflow_execution_requirements()
    }

    type Input = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Input;
    type InputBinding = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::InputBinding;
    type Result = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Result;
    type ResultBinding =
        <RenameBinding as ApplicationMutationBinding<MutationSchema>>::ResultBinding;
    type IdempotencyKey =
        <RenameBinding as ApplicationMutationBinding<MutationSchema>>::IdempotencyKey;
    type Operation = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Operation;
    type Decision = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Decision;
    type Denial = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Denial;
    type DenialBinding =
        <RenameBinding as ApplicationMutationBinding<MutationSchema>>::DenialBinding;
    type Output = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Output;
    type ScopeBinding = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::ScopeBinding;
    type PrincipalBinding =
        <RenameBinding as ApplicationMutationBinding<MutationSchema>>::PrincipalBinding;
    type Mapping = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Mapping;
    type Principal = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::Principal;
    type PrincipalIdentity =
        <RenameBinding as ApplicationMutationBinding<MutationSchema>>::PrincipalIdentity;
    type PrincipalIdentityBinding =
        <RenameBinding as ApplicationMutationBinding<MutationSchema>>::PrincipalIdentityBinding;
    type SourceExpectation =
        <RenameBinding as ApplicationMutationBinding<MutationSchema>>::SourceExpectation;

    const IDENTITY: &'static str = RenameBinding::IDENTITY;
    const HANDLER_IDENTITY: &'static str = RenameBinding::HANDLER_IDENTITY;
    const IDEMPOTENCY_IDENTITY: &'static str = RenameBinding::IDEMPOTENCY_IDENTITY;

    fn scope_field() -> crate::application_schema::ApplicationFieldRef<
        MutationSchema,
        Account,
        AccountIdentity,
        AccountIdField,
        u64,
        crate::application_schema::ReadOnly,
        crate::application_schema::EqualityPredicate,
        crate::application_schema::NoApplicationUnit,
    > {
        RenameBinding::scope_field()
    }

    fn principal_binding() -> crate::application_schema::ApplicationPrincipalBindingRef<
        MutationSchema,
        Self::PrincipalBinding,
        Self::Mapping,
        Self::Principal,
        Self::PrincipalIdentity,
        Self::PrincipalIdentityBinding,
    > {
        RenameBinding::principal_binding()
    }
}

impl crate::application_operation::ApplicationMutationAdmissionBinding<MutationSchema>
    for SameIdentityBinding
{
    type Admission =
        <RenameBinding as crate::application_operation::ApplicationMutationAdmissionBinding<
            MutationSchema,
        >>::Admission;
}

struct OtherOccurrence;
impl ApplicationCompositionInstance for OtherOccurrence {
    const PATH: &'static str = "other-occurrence";
}

struct DuplicateProgram<const CASE: u8>;
impl<const CASE: u8> ApplicationProgramDefinition<MutationSchema> for DuplicateProgram<CASE> {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.action.duplicates.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let source = ApplicationFeatureSpec::root::<MutationSchema, Source>()
            .native_action::<RenameBinding>()
            .finish_action();
        match CASE {
            0 => vec![source
                .native_action::<RenameBinding>()
                .finish_action()
                .finish()],
            1 => vec![source
                .native_action::<SameIdentityBinding>()
                .finish_action()
                .finish()],
            2 => vec![
                source.finish(),
                ApplicationFeatureSpec::at::<MutationSchema, OtherOccurrence, Source>()
                    .native_action::<RenameBinding>()
                    .finish_action()
                    .finish(),
            ],
            3 => vec![ApplicationFeatureSpec::root::<MutationSchema, Source>()
                .native_action::<SameIdentityBinding>()
                .finish_action()
                .finish()],
            _ => unreachable!(),
        }
    }
}

#[test]
fn duplicate_action_refuses_both_native_type_and_portable_resolution_collisions() {
    assert_ne!(
        TypeId::of::<RenameBinding>(),
        TypeId::of::<SameIdentityBinding>()
    );
    assert_eq!(RenameBinding::IDENTITY, SameIdentityBinding::IDENTITY);
    ApplicationProgramAuthoring::<MutationSchema, DuplicateProgram<3>>::begin()
        .validated_program()
        .expect("the second binding is valid without a colliding action");
    for denial in [
        ApplicationProgramAuthoring::<MutationSchema, DuplicateProgram<0>>::begin()
            .validated_program()
            .err()
            .unwrap(),
        ApplicationProgramAuthoring::<MutationSchema, DuplicateProgram<1>>::begin()
            .validated_program()
            .err()
            .unwrap(),
    ] {
        assert_eq!(
            denial.kind(),
            ApplicationProgramValidationDenialKind::DuplicateAction
        );
        assert_eq!(denial.subject(), RenameBinding::IDENTITY);
    }
    let distinct = ApplicationProgramAuthoring::<MutationSchema, DuplicateProgram<2>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(distinct.actions().len(), 2);
    assert_ne!(
        distinct.actions()[0].composition_instance(),
        distinct.actions()[1].composition_instance()
    );
}
