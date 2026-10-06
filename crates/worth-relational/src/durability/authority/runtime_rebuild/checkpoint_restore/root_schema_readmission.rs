use std::collections::BTreeMap;
use std::sync::Arc;

use crate::durability::data::{
    DurabilityError, DurableBranchRootImage, DurableBranchRootSchemaImage, DurableCheckpoint,
};

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
        image: &DurableBranchRootImage,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<Arc<crate::branch::RelationalBranchRootSchemaAuthority>, DurabilityError> {
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
        self.readmit_new_carrier(image, envelope)
    }

    fn readmit_new_carrier(
        &mut self,
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
