//! Stored grants link genuine principals to the model occurrence they govern.
use super::super::schema::*;
use worth_query_decl::facade::application_schema::*;

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
}

pub use crate::proposal_acceptance::schema::storage_model::{
    Grant, GrantAction, GrantDepth, GrantDisclosure, GrantGrantee, GrantGrantor, GrantIdentity,
    GrantNotAfter, GrantNotBefore, GrantParent, GrantPurpose, GrantReadKind, GrantReadStatus,
    GrantResource, GrantScope, GrantStatus,
};
