//! Mutation grants carry no READ-policy flags and have their own lifecycle facts.
use super::super::schema::*;
use worth_query_decl::facade::{application_schema::*, *};
worth_query_entity!(pub MutationGrant for Schema);
worth_query_aspect!(pub MutationGrantFacts for Schema,MutationGrant;identity=AspectIdentity(0x91811708),revision=AspectContractRevision(1));
worth_query_field!(pub GrantIdentity for Schema,MutationGrant,MutationGrantFacts:String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantAction for Schema,MutationGrant,MutationGrantFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantPurpose for Schema,MutationGrant,MutationGrantFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantStatus for Schema,MutationGrant,MutationGrantFacts:u64=>U64ApplicationValueBinding,read_write,equality);
worth_query_field!(pub GrantScope for Schema,MutationGrant,MutationGrantFacts:String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantNotBefore for Schema,MutationGrant,MutationGrantFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantNotAfter for Schema,MutationGrant,MutationGrantFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GrantDepth for Schema,MutationGrant,MutationGrantFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_relation!(pub MutationGrantResource in Schema,MutationGrant=>Model;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub MutationGrantGrantee in Schema,Principal=>MutationGrant;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub MutationGrantGrantor in Schema,Principal=>MutationGrant;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub MutationGrantParent in Schema,MutationGrant=>MutationGrant;integrity=same_context_unbounded_retain_dangling);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(MutationGrant::reference())
        .aspect(MutationGrant::reference(), MutationGrantFacts::reference())
        .field(MutationGrant::reference(), GrantIdentity::reference())
        .field(MutationGrant::reference(), GrantAction::reference())
        .field(MutationGrant::reference(), GrantPurpose::reference())
        .field(MutationGrant::reference(), GrantStatus::reference())
        .field(MutationGrant::reference(), GrantScope::reference())
        .field(MutationGrant::reference(), GrantNotBefore::reference())
        .field(MutationGrant::reference(), GrantNotAfter::reference())
        .field(MutationGrant::reference(), GrantDepth::reference())
        .relation(
            MutationGrantResource::reference(),
            MutationGrant::reference(),
            Model::reference(),
        )
        .relation(
            MutationGrantGrantee::reference(),
            Principal::reference(),
            MutationGrant::reference(),
        )
        .relation(
            MutationGrantGrantor::reference(),
            Principal::reference(),
            MutationGrant::reference(),
        )
        .relation(
            MutationGrantParent::reference(),
            MutationGrant::reference(),
            MutationGrant::reference(),
        )
}
