//! Real grant facts and scope relations consumed by the existing capability owner.
use super::super::super::schema::*;
use worth_query_decl::facade::{application_schema::*, *};
worth_query_entity!(pub BoxGrant for Catalog);
worth_query_aspect!(pub BoxGrantFacts for Catalog, BoxGrant; identity=AspectIdentity(0x919005),revision=AspectContractRevision(1));
worth_query_field!(pub GrantIdentity for Catalog, BoxGrant, BoxGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantAction for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantPurpose for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantStatus for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub GrantReadStatus for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub GrantScope for Catalog, BoxGrant, BoxGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantNotBefore for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantNotAfter for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantDepth for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantDisclosure for Catalog, BoxGrant, BoxGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub GrantResource in Catalog, BoxGrant => Directory; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantee in Catalog, Principal => BoxGrant; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantor in Catalog, Principal => BoxGrant; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantParent in Catalog, BoxGrant => BoxGrant; integrity=same_context_unbounded_retain_dangling);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    schema
        .entity(BoxGrant::reference())
        .aspect(BoxGrant::reference(), BoxGrantFacts::reference())
        .field(BoxGrant::reference(), GrantIdentity::reference())
        .field(BoxGrant::reference(), GrantAction::reference())
        .field(BoxGrant::reference(), GrantPurpose::reference())
        .field(BoxGrant::reference(), GrantStatus::reference())
        .field(BoxGrant::reference(), GrantReadStatus::reference())
        .field(BoxGrant::reference(), GrantScope::reference())
        .field(BoxGrant::reference(), GrantNotBefore::reference())
        .field(BoxGrant::reference(), GrantNotAfter::reference())
        .field(BoxGrant::reference(), GrantDepth::reference())
        .field(BoxGrant::reference(), GrantDisclosure::reference())
        .relation(
            GrantResource::reference(),
            BoxGrant::reference(),
            Directory::reference(),
        )
        .relation(
            GrantGrantee::reference(),
            Principal::reference(),
            BoxGrant::reference(),
        )
        .relation(
            GrantGrantor::reference(),
            Principal::reference(),
            BoxGrant::reference(),
        )
        .relation(
            GrantParent::reference(),
            BoxGrant::reference(),
            BoxGrant::reference(),
        )
}
