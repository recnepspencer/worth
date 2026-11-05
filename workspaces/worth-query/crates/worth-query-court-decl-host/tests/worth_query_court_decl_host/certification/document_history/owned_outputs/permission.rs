//! C can retain a grant to a retired source; the restored model grants nothing.
use super::*;
worth_query_ability!(pub ReadMember scoped_to Member, in Schema);
worth_query_policy!(pub SourceReadPolicy in Schema);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(navigation::grant::Owner::reference())
        .where_equal(
            ModelValue::reference(),
            ApplicationEncodedScalarValue::try_new(1u64).unwrap(),
        )
        .forward(ModelMember::reference())
        .allow(Member::reference());
    schema
        .ability(ReadMember::reference())
        .policy(SourceReadPolicy::reference())
        .ability_policy(
            ReadMember::reference(),
            SourceReadPolicy::reference(),
            [allow],
        )
}
