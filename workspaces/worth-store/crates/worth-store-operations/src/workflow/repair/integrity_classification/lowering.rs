use sha2::{Digest, Sha256};
use worth_proof::{CanonicalVec, NonEmpty};

use super::{
    region::{class_tag, family_tag},
    IntegrityRepairClassificationDenial, IntegrityRepairClassificationPlan,
    IntegrityRepairClassificationReceipt, IntegrityRepairRegion, IntegrityRepairRegionClass,
};

#[derive(Debug, Clone, Copy)]
pub(in crate::workflow::repair) struct IntegrityOperationalRepairOwner;

impl IntegrityOperationalRepairOwner {
    pub(in crate::workflow::repair) fn lower(
        mut regions: Vec<IntegrityRepairRegion>,
    ) -> Result<IntegrityRepairClassificationPlan, IntegrityRepairClassificationDenial> {
        if regions.is_empty() {
            return Err(IntegrityRepairClassificationDenial::EmptyRegions);
        }
        regions.sort();
        if regions
            .windows(2)
            .any(|pair| pair[0].identity() == pair[1].identity())
        {
            return Err(IntegrityRepairClassificationDenial::DuplicateRegion);
        }
        let identities = regions
            .iter()
            .map(|region| region.identity())
            .collect::<Vec<_>>();
        let non_empty = NonEmpty::try_from_vec(identities)
            .map_err(|_| IntegrityRepairClassificationDenial::EmptyRegions)?;
        let mut digest = Sha256::new();
        digest.update(b"worth-store-integrity-repair-classification-plan-v1");
        for region in &regions {
            digest.update(region.identity());
            digest.update(region.start().to_be_bytes());
            digest.update(region.end_exclusive().to_be_bytes());
            digest.update([class_tag(region.class())]);
            digest.update(region.evidence_digest());
            digest.update(region.target_identity());
            let owner = region.owner_binding();
            digest.update([family_tag(owner.family())]);
            digest.update(owner.observed_generation().unwrap_or(0).to_be_bytes());
            digest.update(owner.physical_owner_identity().unwrap_or([0; 32]));
            digest.update(owner.security_scope_identity().unwrap_or([0; 32]));
        }
        Ok(IntegrityRepairClassificationPlan {
            fingerprint: digest.finalize().into(),
            regions: CanonicalVec::try_from_sorted(regions)
                .map_err(|_| IntegrityRepairClassificationDenial::AllocationFailed)?,
            non_empty,
        })
    }

    pub(in crate::workflow::repair) fn execute(
        plan: &IntegrityRepairClassificationPlan,
    ) -> IntegrityRepairClassificationReceipt {
        IntegrityRepairClassificationReceipt {
            plan_fingerprint: plan.fingerprint,
            classified_regions: plan.non_empty.as_slice().len() as u64,
            quarantined_regions: plan
                .regions
                .as_slice()
                .iter()
                .filter(|region| region.class() == IntegrityRepairRegionClass::QuarantineRequired)
                .count() as u64,
        }
    }
}
