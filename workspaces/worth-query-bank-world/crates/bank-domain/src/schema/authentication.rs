use worth_query_decl::facade::application_schema::{fragment::RecordSelection, *};

use super::entities::{ExternalPrincipalMapping, Principal};
use super::relations::ExternalPrincipal;
use super::BankSchema;

pub struct BankPrincipalBinding;
impl BankPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        BankSchema,
        Self,
        ExternalPrincipalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "BankPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityKey::at("ExternalIdentityKey"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    ExternalMappingStatus::at("ExternalMappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    ExternalPrincipal::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

/// The principal, idempotency and default denial every Bank binding macro shares.
///
/// The denial default is the money-movement denial; every other mutation
/// binding states its own `denial`.
pub struct BankBindingContext;
impl ApplicationBindingContext<BankSchema> for BankBindingContext {
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = crate::model::BankPrincipalId;
    type PrincipalIdentityBinding = super::BankPrincipalIdBinding;
    type IdempotencyKey = crate::proposals::BankIdempotencyKey;
    type DenialBinding = super::MoneyMovementDenialBinding;

    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BankSchema,
        BankPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        crate::model::BankPrincipalId,
        super::BankPrincipalIdBinding,
    > {
        BankPrincipalBinding::reference()
    }
}

pub use crate::schema::entities::storage_model::{
    ExternalIdentityKey, ExternalMappingStatus, ExternalPrincipalIdentity, PrincipalIdentity,
    PrincipalIdentityField,
};
