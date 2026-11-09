//! A correction source may already be another action's declared inverse binding.
use super::*;

#[test]
fn correction_source_reuses_the_complete_preinstalled_binding() {
    let declaration =
        ApplicationSchemaDeclarationBuilder::<MutationSchema>::from_fragment(ReuseFragment)
            .unwrap()
            .build()
            .expect("an exact preinstalled inverse can also become a correction source");
    assert_eq!(
        declaration.member_provenance().mutation_bindings(),
        &[RenameBinding::descriptor()],
    );
    assert_eq!(
        declaration
            .erased()
            .members()
            .iter()
            .filter(|member| matches!(member, ApplicationSchemaMember::ApplicationMutation { .. }))
            .count(),
        1,
    );
}

#[test]
fn correction_source_does_not_hide_a_conflicting_portable_scope() {
    let descriptor = RenameBinding::descriptor();
    let mut conflicting = descriptor.description().clone().into_parts();
    conflicting.scope.field = "DifferentScopeField".to_owned();
    let member = ApplicationSchemaMember::ApplicationMutation {
        description: ApplicationMutationDescription::from_untrusted_parts(conflicting),
    };
    let builder =
        ApplicationSchemaDeclarationBuilder::<MutationSchema>::from_test_members(vec![member])
            .application_mutation_binding::<RenameBinding>();
    let outcome = builder.correction::<Inverse>().build();
    assert_eq!(
        outcome.unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}

struct ReuseFragment;
impl Fragment for ReuseFragment {
    const NAME: &'static str = "source_reuse";
    type Bindings = NoBindings;
    type Members = (Preinstalled, CorrectionMembers);
}
struct Preinstalled;
impl Registration<MutationSchema> for Preinstalled {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<MutationSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<MutationSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<RenameBinding>())
    }
}
