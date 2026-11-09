//! Mutation grants carry no READ-policy flags and have their own lifecycle facts.
use super::super::schema::*;
use worth_query_decl::facade::application_schema::*;

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
}

pub use crate::proposal_acceptance::schema::storage_model::{
    MutationGrant, MutationGrantAction as GrantAction, MutationGrantDepth as GrantDepth,
    MutationGrantGrantee, MutationGrantGrantor, MutationGrantIdentity as GrantIdentity,
    MutationGrantNotAfter as GrantNotAfter, MutationGrantNotBefore as GrantNotBefore,
    MutationGrantParent, MutationGrantPurpose as GrantPurpose, MutationGrantResource,
    MutationGrantScope as GrantScope, MutationGrantStatus as GrantStatus,
};
