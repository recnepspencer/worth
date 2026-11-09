//! The actual ability observer reads its own live grant, independent of facets.
pub use super::super::grants::*;
pub use worth_query_decl::facade::{worth_query_ability, worth_query_policy};

worth_query_ability!(pub AdvanceResource scoped_to Document, in CapabilitySchema);
worth_query_policy!(pub ResourceActionPolicy in CapabilitySchema);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
) -> ApplicationSchemaDeclarationBuilder<CapabilitySchema> {
    schema
        .ability(AdvanceResource::reference())
        .policy(ResourceActionPolicy::reference())
        .ability_policy(
            AdvanceResource::reference(),
            ResourceActionPolicy::reference(),
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .forward(GrantGrantee::reference())
                    .where_equal(
                        GrantIdentity::at("GrantIdentity"),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            "resource-action".to_owned(),
                        )
                        .unwrap(),
                    )
                    .unwrap()
                    .where_equal(
                        GrantStatus::at("GrantStatus"),
                        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1)
                            .unwrap(),
                    )
                    .unwrap()
                    .forward(GrantResource::reference())
                    .allow(Document::reference()),
            ],
        )
}
