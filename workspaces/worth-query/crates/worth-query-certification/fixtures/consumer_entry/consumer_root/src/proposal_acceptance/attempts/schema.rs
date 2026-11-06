//! The model links issued claim and result occurrences on its stable storage product.
use super::super::schema::*;
use worth_query_decl::facade::application_schema::*;

pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    builder
}

pub use crate::proposal_acceptance::schema::storage_model::{
    AcceptanceClaim, AcceptanceResult, ClaimData, ClaimName, ModelClaim, ModelResult, ResultData,
    ResultName,
};
