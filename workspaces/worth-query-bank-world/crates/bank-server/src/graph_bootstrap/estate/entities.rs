use bank_domain::{estate::BankEstateWorld, schema::*};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntitySeed, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};

use super::{
    super::entity_key,
    keys::{authority, branch, emergency, estate, grant, notice, review},
};

pub(super) fn bind(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    world: &BankEstateWorld,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    bind_core(graph, world)?;
    bind_capabilities(graph, world)?;
    bind_emergency_access(graph, world)?;
    bind_reviews(graph, world)
}

fn bind_core(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    world: &BankEstateWorld,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for value in world.branches() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Branch::reference(),
                entity_key(branch(value.id.get())),
            )
            .field(BranchIdentityField::at("BranchIdentityField"), value.id),
        )?;
    }
    for value in world.death_notices() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                DeathNotice::reference(),
                entity_key(notice(value.id.get())),
            )
            .field(
                DeathNoticeIdentityField::at("DeathNoticeIdentityField"),
                value.id,
            )
            .field(
                DeathNoticeStatusField::at("DeathNoticeStatusField"),
                value.status,
            ),
        )?;
    }
    for value in world.cases() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                EstateCase::reference(),
                entity_key(estate(value.id.get())),
            )
            .field(
                EstateCaseIdentityField::at("EstateCaseIdentityField"),
                value.id,
            )
            .field(
                EstateWorkflowStageField::at("EstateWorkflowStageField"),
                value.stage,
            )
            .field(
                EstateCaseStatusField::at("EstateCaseStatusField"),
                value.status,
            ),
        )?;
    }
    for value in world.legal_authorities() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                LegalAuthority::reference(),
                entity_key(authority(value.id.get())),
            )
            .field(
                LegalAuthorityIdentityField::at("LegalAuthorityIdentityField"),
                value.id,
            )
            .field(
                LegalAuthorityKindField::at("LegalAuthorityKindField"),
                value.kind,
            )
            .field(
                LegalAuthorityRecognizedField::at("LegalAuthorityRecognizedField"),
                value.recognized,
            ),
        )?;
    }
    Ok(())
}

fn bind_capabilities(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    world: &BankEstateWorld,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for value in world.grants() {
        let mut seed = WorthQueryApplicationEntitySeed::new(
            CapabilityGrant::reference(),
            entity_key(grant(value.id.get())),
        )
        .field(
            CapabilityGrantIdentityField::at("CapabilityGrantIdentityField"),
            value.id,
        )
        .field(
            CapabilityOperationField::at("CapabilityOperationField"),
            value.scope.operation,
        )
        .field(
            CapabilityPurposeField::at("CapabilityPurposeField"),
            value.scope.purpose,
        )
        .field(
            CapabilityValidFromField::at("CapabilityValidFromField"),
            value.scope.validity.not_before(),
        )
        .field(
            CapabilityValidThroughField::at("CapabilityValidThroughField"),
            value.scope.validity.not_after(),
        )
        .field(
            CapabilityDelegationLimitField::at("CapabilityDelegationLimitField"),
            value.scope.delegation,
        )
        .field(
            CapabilityWorkflowStageField::at("CapabilityWorkflowStageField"),
            value.scope.workflow_stage,
        )
        .field(
            CapabilityGrantStatusField::at("CapabilityGrantStatusField"),
            value.status,
        );
        if let Some(field) = value.scope.field {
            seed = seed.field(
                CapabilityDisclosureField::at("CapabilityDisclosureField"),
                field,
            );
        }
        if let Some(amount) = value.scope.amount_ceiling {
            seed = seed.field(
                CapabilityAmountCeilingField::at("CapabilityAmountCeilingField"),
                amount,
            );
        }
        graph.bind_entity(seed)?;
    }
    Ok(())
}

fn bind_emergency_access(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    world: &BankEstateWorld,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for value in world.emergency_accesses() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                EmergencyAccess::reference(),
                entity_key(emergency(value.id.get())),
            )
            .field(
                EmergencyAccessIdentityField::at("EmergencyAccessIdentityField"),
                value.id,
            )
            .field(
                EmergencyAccessReasonField::at("EmergencyAccessReasonField"),
                value.reason,
            )
            .field(
                EmergencyAccessStatusField::at("EmergencyAccessStatusField"),
                value.status,
            )
            .field(
                EmergencyAccessIssuedAtField::at("EmergencyAccessIssuedAtField"),
                value.issued_at,
            )
            .field(
                EmergencyAccessExpiresAtField::at("EmergencyAccessExpiresAtField"),
                value.expires_at,
            ),
        )?;
    }
    Ok(())
}

fn bind_reviews(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    world: &BankEstateWorld,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for value in world.reviews() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                MandatoryReview::reference(),
                entity_key(review(value.id.get())),
            )
            .field(
                MandatoryReviewIdentityField::at("MandatoryReviewIdentityField"),
                value.id,
            )
            .field(
                MandatoryReviewKindField::at("MandatoryReviewKindField"),
                value.kind,
            )
            .field(
                MandatoryReviewStatusField::at("MandatoryReviewStatusField"),
                value.status,
            ),
        )?;
    }
    Ok(())
}
