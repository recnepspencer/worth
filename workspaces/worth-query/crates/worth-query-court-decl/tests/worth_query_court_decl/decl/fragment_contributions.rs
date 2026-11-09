//! Contribution closure includes derived columns only from its actual authored requests.
use worth_query_testkit_decl::fragment_contribution_fixture::*;
#[test]
fn contribution_members_own_deferred_record_reads_writes_and_creation_presence() {
    let declaration = Composed::declaration().unwrap();
    let [contribution] = declaration.contributions() else {
        panic!("one contribution")
    };
    assert_eq!(contribution.identity().as_str(), "shared");
    assert_eq!(
        contribution.member_ordinals().len(),
        declaration.erased().members().len()
    );
    let title: FieldSelection<Model, Entry, Payload, String> =
        select_field(Model, Entry, Payload, "title");
    assert!(declaration.erased().members().iter().any(|member| matches!(member,
        ApplicationSchemaMember::OperationProgram { target: ApplicationOperationProgramTarget::Write { field, .. }, .. } if field == title.field())));
}
#[test]
fn root_record_requests_cannot_borrow_a_contributions_ownership() {
    assert!(matches!(
        Incomplete::declaration(),
        Err(ApplicationSchemaDeclarationDenial::IncompleteContributionClosure)
    ));
}
#[test]
fn duplicate_contribution_members_receive_a_typed_build_refusal() {
    assert!(matches!(
        Duplicate::declaration(),
        Err(ApplicationSchemaDeclarationDenial::DuplicateContributionIdentity)
    ));
}
