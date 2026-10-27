//! The actual ability observer reads its own live grant, independent of facets.
use super::super::{grants::*, schema::*};
use worth_query_decl::facade::{application_schema::*, worth_query_ability, worth_query_policy};

worth_query_ability!(pub AdvanceResource scoped_to Document, in CapabilitySchema);
worth_query_policy!(pub ResourceActionPolicy in CapabilitySchema);

pub(super) fn declare(
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
                        GrantIdentity::reference(),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            "resource-action".to_owned(),
                        )
                        .unwrap(),
                    )
                    .where_equal(
                        GrantStatus::reference(),
                        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1)
                            .unwrap(),
                    )
                    .forward(GrantResource::reference())
                    .allow(Document::reference()),
            ],
        )
}
