use super::*;
use crate::physical_runtime::durability::{PhysicalRootPublicationIdentity, RetiredArtifact};
use crate::physical_runtime::record_serving::planning::rebased_root::{
    plan_arena_forget, plan_retirement_release, RetirementRootPlanningContext,
};
use worth_store_physical_format::RecordArtifactFile;

impl RecordPublicationDirector {
    pub(in super::super) fn prepare_extent_release(
        &self,
        displaced: DisplacedArtifact,
    ) -> Result<(), PhysicalRetirementDenial> {
        if matches!(displaced.artifact, RetiredArtifact::Segment { .. }) {
            return Err(PhysicalRetirementDenial::Retained);
        }
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalRetirementDenial::Unresolved)?;
        let reserved = self
            .mutation_identity
            .reserve_mutation_identity()
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let operation = crate::physical_runtime::PhysicalMutationIdentity::from_reserved_operation(
            reserved.identity(),
        );
        let pending = self
            .root_owner
            .publication_admission()
            .register_exclusive_pending(operation)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        if let Some(denial) = self
            .root_owner
            .blocked_retirement_release(&displaced, &pending)
        {
            return Err(denial);
        }
        if let RetiredArtifact::Extent { range, .. } = displaced.artifact {
            let preparation = self.preparation.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(owner) = &preparation.arenas {
                owner
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .preflight_release(range)
                    .map_err(|_| PhysicalRetirementDenial::Waiting)?;
            }
        }
        let (source, free) = self.root_owner.snapshot();
        let height = free.root().map_or(0, |root| u64::from(root.level()));
        let bytes = u64::from(self.format.declaration().page_size().bytes())
            .checked_mul(6 * (height + 1) + 8)
            .and_then(NonZeroU64::new)
            .ok_or(PhysicalRetirementDenial::Waiting)?;
        let allocation = self
            .residency
            .begin_foreground_write_operation(bytes)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let publication = super::super::super::append::next_nonzero_random()
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let context = RetirementRootPlanningContext {
            allocation: &allocation,
            residency: self.residency.clone(),
            format: self.format,
            access: self.access,
            media: runtime.executor.record_serving_media(),
            current_root: &source,
            current_free: &free,
        };
        let candidate = RecordArtifactFile::CatalogCandidate { publication };
        let (plan, free) = match displaced.artifact {
            RetiredArtifact::Extent { range, .. } => {
                plan_retirement_release(context, range, candidate)
            }
            RetiredArtifact::Arena { arena, .. } => plan_arena_forget(
                context,
                worth_store_physical_format::ExtentArenaId::new(arena)
                    .ok_or(PhysicalRetirementDenial::WalPlan)?,
                candidate,
            ),
            RetiredArtifact::Segment { .. } => {
                unreachable!("segment retirement has no free-map publication")
            }
        }
        .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let growth_bytes = plan
            .retained_metadata_bytes()
            .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let release = RetirementReleaseProjection::new(
            source.generation(),
            plan.generation,
            Sha256::digest(&plan.root_bytes).into(),
            growth_bytes,
            publication,
        )
        .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let intent = encode_retirement(
            displaced.artifact,
            false,
            displaced.source_root,
            displaced.bytes,
            Some(release),
        );
        let identity = PhysicalRootPublicationIdentity::from_retirement(
            self.durability.policy_identity(),
            operation,
            release,
            Sha256::digest(&intent).into(),
            publication,
        )
        .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let transition = self
            .root_owner
            .begin(identity, source.clone())
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let growth = self
            .root_owner
            .publication_admission()
            .reserve_retained_bytes(growth_bytes)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let mut state = self
            .retirement_release
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.is_some() {
            return Err(PhysicalRetirementDenial::Unresolved);
        }
        if let RetiredArtifact::Arena { arena, .. } = displaced.artifact {
            self.expose_arena_retirement(arena)?;
        }
        *state = Some(PendingRetirementRelease {
            displaced,
            release,
            pending: Some(pending),
            intent,
            growth: Some(growth),
            growth_bytes,
            progress: ReleaseProgress::Prepared(ReleaseCandidate {
                source,
                free,
                plan,
                transition,
                allocation,
                retry: None,
            }),
        });
        Ok(())
    }
}
