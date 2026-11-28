//! Derived declarations, including explicit wire-identity overrides, lower
//! to the same canonical schema as the same members written by hand.
use super::schema::{
    Board, BodyDenialBinding, ExternalIdentityField, ExternalMapping, MappingStatus, MappingTarget,
    Principal, PrincipalBinding, PrincipalIdentity, Schema, BOARD,
};
use worth_query_decl::facade::application_schema::fragment::{relation, Registration};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::worth_query_identity;

mod action;
mod body_and_scope;
mod collapsed_program;
mod ergonomics_edge_cases;
mod host_negative_ergonomics;
mod hygiene_and_typed_input_courts;
mod projection;
mod query;
mod query_binding_conflict;
mod record;
mod relation;

fn board() -> ApplicationSchemaDeclarationBuilder<Schema> {
    ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Board)
        .expect("the board fragment lowers")
        .principal_binding(PrincipalBinding::reference())
}

/// Schema-parametric principal binding reusing the `Board` fragment for
/// standalone negative/conflict schemas.
pub struct GenericBoardPrincipal;
impl GenericBoardPrincipal {
    pub fn reference<S: ApplicationSchema>() -> ApplicationPrincipalBindingRef<
        S,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                    Board,
                    ExternalMapping,
                    MappingTarget,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

impl<S: ApplicationSchema> Registration<S> for GenericBoardPrincipal {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        Ok(builder.principal_binding(Self::reference()))
    }
}

/// Schema-parametric binding context for standalone `Board`-based schemas.
pub struct GenericBoardContext;
impl<S: ApplicationSchema> ApplicationBindingContext<S> for GenericBoardContext {
    type PrincipalBinding = GenericBoardPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = worth_query_identity!(BOARD, idempotency);

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        S,
        GenericBoardPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        GenericBoardPrincipal::reference()
    }
}
