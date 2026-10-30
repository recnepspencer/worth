//! Stored grants link genuine principals to the model occurrence they govern.
use super::super::schema::*;
use worth_query_decl::facade::{application_schema::*, *};

worth_query_entity!(pub Grant for Schema);
worth_query_aspect!(pub GrantFacts for Schema, Grant; identity=AspectIdentity(0x91811707),revision=AspectContractRevision(1));
worth_query_field!(pub GrantIdentity for Schema, Grant, GrantFacts: String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantAction for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantReadKind for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantPurpose for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantStatus for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_write,equality);
worth_query_field!(pub GrantReadStatus for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_write,equality);
worth_query_field!(pub GrantScope for Schema, Grant, GrantFacts: String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantNotBefore for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantNotAfter for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantDepth for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantDisclosure for Schema, Grant, GrantFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_relation!(pub GrantResource in Schema, Grant=>Model;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantee in Schema, Principal=>Grant;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantor in Schema, Principal=>Grant;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantParent in Schema, Grant=>Grant;integrity=same_context_unbounded_retain_dangling);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Grant::reference())
        .aspect(Grant::reference(), GrantFacts::reference())
        .field(Grant::reference(), GrantIdentity::reference())
        .field(Grant::reference(), GrantAction::reference())
        .field(Grant::reference(), GrantReadKind::reference())
        .field(Grant::reference(), GrantPurpose::reference())
        .field(Grant::reference(), GrantStatus::reference())
        .field(Grant::reference(), GrantReadStatus::reference())
        .field(Grant::reference(), GrantScope::reference())
        .field(Grant::reference(), GrantNotBefore::reference())
        .field(Grant::reference(), GrantNotAfter::reference())
        .field(Grant::reference(), GrantDepth::reference())
        .field(Grant::reference(), GrantDisclosure::reference())
        .relation(
            GrantResource::reference(),
            Grant::reference(),
            Model::reference(),
        )
        .relation(
            GrantGrantee::reference(),
            Principal::reference(),
            Grant::reference(),
        )
        .relation(
            GrantGrantor::reference(),
            Principal::reference(),
            Grant::reference(),
        )
        .relation(
            GrantParent::reference(),
            Grant::reference(),
            Grant::reference(),
        )
}
