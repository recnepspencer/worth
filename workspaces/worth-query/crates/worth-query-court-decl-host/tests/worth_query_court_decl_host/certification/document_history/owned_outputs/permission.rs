//! C can retain a grant to a retired source; the restored model grants nothing.
use super::*;
worth_query_ability!(pub ReadMember scoped_to Member, in Schema);
worth_query_policy!(pub SourceReadPolicy in Schema);
worth_query_ability!(pub ReadFirst scoped_to FirstOutput, in Schema);
worth_query_policy!(pub FirstReadPolicy in Schema);
worth_query_ability!(pub ReadFirstWithMember scoped_to FirstOutput, in Schema);
worth_query_policy!(pub FirstMemberReadPolicy in Schema);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(navigation::grant::Owner::reference())
        .where_equal(
            ModelValue::at("ModelValue"),
            ApplicationEncodedScalarValue::try_new(1u64).unwrap(),
        )
        .unwrap()
        .forward(ModelMember::reference())
        .allow(Member::reference());
    let first_member = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(navigation::grant::Owner::reference())
        .forward(ModelMember::reference())
        .where_equal(
            MemberValue::at("MemberValue"),
            ApplicationEncodedScalarValue::try_new(5u64).unwrap(),
        )
        .unwrap()
        .reverse(ModelMember::reference())
        .reverse(FirstOf::reference())
        .allow(FirstOutput::reference());
    schema
        .ability(ReadFirstWithMember::reference())
        .policy(FirstMemberReadPolicy::reference())
        .ability_policy(
            ReadFirstWithMember::reference(),
            FirstMemberReadPolicy::reference(),
            [first_member],
        )
        .ability(ReadFirst::reference())
        .policy(FirstReadPolicy::reference())
        .ability_policy(
            ReadFirst::reference(),
            FirstReadPolicy::reference(),
            [first_path(1), first_path(3), first_path(5)],
        )
        .ability(ReadMember::reference())
        .policy(SourceReadPolicy::reference())
        .ability_policy(
            ReadMember::reference(),
            SourceReadPolicy::reference(),
            [allow],
        )
}

fn first_path(value: u64) -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(navigation::grant::Owner::reference())
        .where_equal(
            ModelValue::at("ModelValue"),
            ApplicationEncodedScalarValue::try_new(value).unwrap(),
        )
        .unwrap()
        .reverse(FirstOf::reference())
        .allow(FirstOutput::reference())
}
