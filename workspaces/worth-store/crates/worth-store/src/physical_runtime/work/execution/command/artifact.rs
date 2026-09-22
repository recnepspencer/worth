use sha2::{Digest, Sha256};
use worth_store_physical_format::RecordFrameCoordinate;

use super::super::super::{PhysicalWorkOperationFamily, ResourceAdmittedPhysicalWork};
use super::types::{
    require_family, PhysicalExecutorCommand, PhysicalExecutorCommandDenial,
    PhysicalMetadataExecutorCommand, PhysicalPublicationEffect, PhysicalPublicationExecutorCommand,
    PhysicalReadExecutorCommand, PhysicalWriteExecutorCommand,
};

impl PhysicalExecutorCommand {
    pub fn metadata(
        work: ResourceAdmittedPhysicalWork,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::ArtifactMetadataRead)?;
        let artifact = work
            .intent()
            .scope()
            .artifact_target()
            .ok_or(PhysicalExecutorCommandDenial::ArtifactCommandRequiresArtifactScope)?;
        Ok(Self::Metadata(PhysicalMetadataExecutorCommand {
            work,
            artifact,
        }))
    }

    pub fn read(work: ResourceAdmittedPhysicalWork) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::ArtifactRangeRead)?;
        let coordinate = exact_coordinate(&work)?;
        Ok(Self::Read(PhysicalReadExecutorCommand {
            work,
            coordinate,
            destination: vec![0_u8; coordinate.length() as usize].into_boxed_slice(),
        }))
    }

    pub fn exact_write(
        work: ResourceAdmittedPhysicalWork,
        payload: impl Into<Box<[u8]>>,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::ArtifactRangeWrite)?;
        Ok(Self::ExactWrite(PhysicalWriteExecutorCommand::new(
            work, payload,
        )?))
    }

    pub fn publication(
        work: ResourceAdmittedPhysicalWork,
        payload: impl Into<Box<[u8]>>,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::ArtifactPublication)?;
        Ok(Self::Publication(PhysicalWriteExecutorCommand::new(
            work, payload,
        )?))
    }

    pub fn new_artifact(
        work: ResourceAdmittedPhysicalWork,
        payload: impl Into<Box<[u8]>>,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::ArtifactPublication)?;
        Ok(Self::NewArtifact(PhysicalWriteExecutorCommand::new(
            work, payload,
        )?))
    }

    pub fn publication_effect(
        work: ResourceAdmittedPhysicalWork,
        effect: PhysicalPublicationEffect,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::ArtifactPublication)?;
        let artifact = work
            .intent()
            .scope()
            .artifact_target()
            .ok_or(PhysicalExecutorCommandDenial::ArtifactCommandRequiresArtifactScope)?;
        if matches!(effect, PhysicalPublicationEffect::ReplaceCatalog)
            && !matches!(
                artifact,
                worth_store_physical_format::RecordArtifactFile::CatalogCandidate { .. }
            )
        {
            return Err(PhysicalExecutorCommandDenial::ArtifactCommandRequiresArtifactScope);
        }
        if matches!(effect, PhysicalPublicationEffect::SynchronizeRecordFamily)
            && artifact != worth_store_physical_format::RecordArtifactFile::BootstrapCatalog
        {
            return Err(PhysicalExecutorCommandDenial::ArtifactCommandRequiresArtifactScope);
        }
        if matches!(effect, PhysicalPublicationEffect::RemoveArtifact) {
            return Err(PhysicalExecutorCommandDenial::RetirementRemovalRequiresClaim);
        }
        Ok(Self::PublicationEffect(
            PhysicalPublicationExecutorCommand {
                work,
                artifact,
                effect,
            },
        ))
    }

    pub(in crate::physical_runtime) fn retirement_removal(
        work: ResourceAdmittedPhysicalWork,
        permit: crate::physical_runtime::durability::RetirementRemovalPermit,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::ArtifactPublication)?;
        let artifact = work
            .intent()
            .scope()
            .artifact_target()
            .ok_or(PhysicalExecutorCommandDenial::ArtifactCommandRequiresArtifactScope)?;
        let worth_store_physical_format::RecordArtifactFile::Segment {
            segment,
            generation,
        } = artifact
        else {
            return Err(PhysicalExecutorCommandDenial::RetirementRemovalRequiresClaim);
        };
        if segment != permit.segment_id() || generation != permit.generation() {
            return Err(PhysicalExecutorCommandDenial::RetirementRemovalRequiresClaim);
        }
        Ok(Self::PublicationEffect(
            PhysicalPublicationExecutorCommand {
                work,
                artifact,
                effect: PhysicalPublicationEffect::RemoveArtifact,
            },
        ))
    }
}

impl PhysicalWriteExecutorCommand {
    fn new(
        work: ResourceAdmittedPhysicalWork,
        payload: impl Into<Box<[u8]>>,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        let coordinate = exact_coordinate(&work)?;
        let payload = payload.into();
        if payload.len() != coordinate.length() as usize {
            return Err(PhysicalExecutorCommandDenial::PayloadLengthMismatch);
        }
        let payload_digest = Sha256::digest(&payload).into();
        Ok(Self {
            work,
            coordinate,
            payload,
            payload_digest,
        })
    }
}

fn exact_coordinate(
    work: &ResourceAdmittedPhysicalWork,
) -> Result<RecordFrameCoordinate, PhysicalExecutorCommandDenial> {
    let [coordinate] = work.intent().scope().coordinates() else {
        return Err(PhysicalExecutorCommandDenial::ExactCommandRequiresOneRange);
    };
    Ok(*coordinate)
}
