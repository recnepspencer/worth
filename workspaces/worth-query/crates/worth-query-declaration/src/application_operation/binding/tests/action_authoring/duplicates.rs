//! Portable resolution identity and native type identity share one validator.
use super::*;

crate::worth_query_mutation_binding!(
    SameIdentityBinding for RenameInput, schema MutationSchema,
    identity "worth.query.test.rename-binding.v1",
    input RenameInputBinding, operation RenameOperation, result RenameResultBinding,
    idempotency u64, identity "worth.query.test.rename-idempotency.v1",
    decision RenameDecision, denial RenameDenialBinding,
    handler identity "worth.query.test.rename-handler.v1",
    outputs RenameOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Account,
        principal_identity u64, identity_binding crate::application_schema::U64ApplicationValueBinding,
    scope Account, AccountIdentity, AccountIdField, u64,
        crate::application_schema::ReadOnly, crate::application_schema::NoApplicationUnit,
    field AccountIdField::reference(), value rename_scope,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 1,
    resources retained_representation_bytes 128
);

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
            0 => vec![
                source
                    .native_action::<RenameBinding>()
                    .finish_action()
                    .finish(),
            ],
            1 => vec![
                source
                    .native_action::<SameIdentityBinding>()
                    .finish_action()
                    .finish(),
            ],
            2 => vec![
                source.finish(),
                ApplicationFeatureSpec::at::<MutationSchema, OtherOccurrence, Source>()
                    .native_action::<RenameBinding>()
                    .finish_action()
                    .finish(),
            ],
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
