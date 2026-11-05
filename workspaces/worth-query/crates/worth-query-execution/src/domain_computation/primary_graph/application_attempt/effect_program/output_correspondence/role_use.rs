//! The one internal form of an output-role use. Every public use site builds
//! it from a declaration marker through [`OutputRoleUse::fixed`] or
//! [`OutputRoleUse::member`], and both evaluate the marker's contract check
//! in an inline `const` block, so a use its contract does not declare fails
//! to compile at the instantiating call.

use crate::equality_type_id::EqualityTypeId as TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputPosture, ApplicationMutationOutputRoleCardinality,
    WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationDeclaredOutputRoleFamily,
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputCardinality,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily,
    WorthQueryApplicationOutputRoleNameDenial,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;




mod court_doors_for_test;
