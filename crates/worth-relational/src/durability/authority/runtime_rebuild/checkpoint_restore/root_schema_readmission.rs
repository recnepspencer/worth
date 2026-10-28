use std::collections::BTreeMap;
use std::sync::Arc;

use crate::durability::data::{
    DurabilityError, DurableBranchRootImage, DurableBranchRootSchemaImage, DurableCheckpoint,
};
use crate::runtime::RelationalRuntime;

type ReadmittedCarrier = (
    DurableBranchRootSchemaImage,
    crate::schema::data::RelationalSchemaRegistry,
    Vec<worth_foundational::facade::AspectContract>,
);

pub(super) struct RootSchemaReadmissionCatalog {
    carriers: BTreeMap<[u8; 32], ReadmittedCarrier>,
    admitted: BTreeMap<[u8; 32], Arc<crate::branch::RelationalBranchRootSchemaAuthority>>,
    next_authority_id: u64,
}

impl RootSchemaReadmissionCatalog {
    pub(super) fn readmit(checkpoint: &DurableCheckpoint) -> Result<Self, DurabilityError> {
        let mut carriers = BTreeMap::new();
        for image in &checkpoint.branch_root_schema_images {
            let (registry, retained_contracts) =
                image.readmit_registry().map_err(corrupt_checkpoint)?;
            if carriers
                .insert(
                    image.carrier_digest(),
                    (image.clone(), registry, retained_contracts),
                )
                .is_some()
            {
                return Err(corrupt_checkpoint(
                    "duplicate branch-root schema carrier digest".to_owned(),
                ));
            }
        }
        Ok(Self {
            carriers,
            admitted: BTreeMap::new(),
            next_authority_id: 1,
        })
    }

    pub(super) fn readmit_root(
        &mut self,
        restored: &RelationalRuntime,
        image: &DurableBranchRootImage,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<Arc<crate::branch::RelationalBranchRootSchemaAuthority>, DurabilityError> {
        if image.format_version == 0 {
            return self.readmit_legacy_root(restored, image, envelope);
        }
        validate_root_image_integrity(image)?;
        if let Some(authority) = self.admitted.get(&image.schema_carrier_digest) {
            if !authority.matches(&envelope.schema_authority)
                || authority.descriptor_semantics_version() != envelope.descriptor_semantics_version
            {
                return Err(corrupt_checkpoint(
                    "shared schema carrier disagrees with a linked envelope".to_owned(),
                ));
            }
            return Ok(Arc::clone(authority));
        }
        self.readmit_new_carrier(restored, image, envelope)
    }

    fn readmit_legacy_root(
        &mut self,
        restored: &RelationalRuntime,
        image: &DurableBranchRootImage,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<Arc<crate::branch::RelationalBranchRootSchemaAuthority>, DurabilityError> {
        let allocation_id = self.issue_authority_id()?;
        crate::branch::RelationalBranchRootSchemaAuthority::readmit_exact(
            allocation_id,
            restored.config.schema.registry.clone(),
            active_contracts(&restored.schema_contract_runtime.aspect_contract_plans),
            &envelope.schema_authority,
            envelope.descriptor_semantics_version,
        )
        .ok_or_else(|| {
            schema_mismatch(format!(
                "legacy branch-root image `{}` has no exact live schema authority",
                image.commit_id.0
            ))
        })
    }

    fn readmit_new_carrier(
        &mut self,
        restored: &RelationalRuntime,
        image: &DurableBranchRootImage,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<Arc<crate::branch::RelationalBranchRootSchemaAuthority>, DurabilityError> {
        let (carrier, registry, retained_contracts) = self
            .carriers
            .get(&image.schema_carrier_digest)
            .cloned()
            .ok_or_else(|| {
                corrupt_checkpoint(format!(
                    "branch-root image `{}` references a missing schema carrier",
                    image.commit_id.0
                ))
            })?;
        let expected_root =
            crate::schema::data::schema_authority_snapshot_digest_bytes(&envelope.schema_authority);
        if carrier.schema_root() != expected_root
            || carrier.descriptor_semantics_version() != envelope.descriptor_semantics_version
        {
            return Err(corrupt_checkpoint(format!(
                "branch-root image `{}` schema carrier linkage mismatch",
                image.commit_id.0
            )));
        }
        // Carrier validation alone cannot substitute the configured source
        // issuer. First bind the complete durable meaning to this envelope.
        if registry.authority_snapshot() != envelope.schema_authority {
            return Err(schema_mismatch(format!(
                "branch-root image `{}` executable schema differs from its envelope",
                image.commit_id.0
            )));
        }
        let configured = &restored.config.schema.registry;
        let registry = if (!envelope.schema_authority.derived_relations.is_empty()
            || !envelope
                .schema_authority
                .projected_member_domains
                .is_empty())
            && configured.authority_snapshot() == envelope.schema_authority
        {
            // Installed typed rules/layouts already retain these genuine entry
            // handles. Exact admitted meaning preserves their original issuer;
            // a changed configured recipe must keep the carrier's own registry.
            configured.clone()
        } else {
            registry
        };
        let allocation_id = self.issue_authority_id()?;
        let authority = crate::branch::RelationalBranchRootSchemaAuthority::readmit_exact(
            allocation_id,
            registry,
            retained_contracts,
            &envelope.schema_authority,
            envelope.descriptor_semantics_version,
        )
        .ok_or_else(|| {
            schema_mismatch(format!(
                "branch-root image `{}` executable schema differs from its envelope",
                image.commit_id.0
            ))
        })?;
        self.admitted
            .insert(image.schema_carrier_digest, Arc::clone(&authority));
        Ok(authority)
    }

    fn issue_authority_id(&mut self) -> Result<u64, DurabilityError> {
        let issued = self.next_authority_id;
        self.next_authority_id = self
            .next_authority_id
            .checked_add(1)
            .ok_or_else(|| corrupt_checkpoint("schema authority identity exhausted".to_owned()))?;
        Ok(issued)
    }
}

fn validate_root_image_integrity(image: &DurableBranchRootImage) -> Result<(), DurabilityError> {
    if image.format_version != DurableBranchRootImage::CURRENT_FORMAT_VERSION {
        return Err(corrupt_checkpoint(format!(
            "unsupported branch-root image version `{}`",
            image.format_version
        )));
    }
    let digest = crate::durability::data::branch_root_image_digest(
        image.format_version,
        image.commit_id,
        image.partition_image_digest,
        image.schema_carrier_digest,
    );
    if digest == image.root_image_digest {
        Ok(())
    } else {
        Err(corrupt_checkpoint(format!(
            "branch-root image `{}` aggregate integrity mismatch",
            image.commit_id.0
        )))
    }
}

fn active_contracts(
    plans: &crate::schema::data::AspectContractPlanCatalog,
) -> Vec<worth_foundational::facade::AspectContract> {
    let mut contracts = BTreeMap::new();
    for binding in plans
        .entity_plans
        .values()
        .chain(plans.relation_plans.values())
        .flat_map(|plan| &plan.executable_bindings)
    {
        contracts.insert(
            (
                binding.contract.key().clone(),
                binding.contract.identity(),
                binding.contract.revision(),
            ),
            binding.contract.clone(),
        );
    }
    contracts.into_values().collect()
}

fn corrupt_checkpoint(detail: String) -> DurabilityError {
    DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
        detail,
    )
}

fn schema_mismatch(detail: String) -> DurabilityError {
    DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::SchemaMismatch,
        detail,
    )
}
