use super::*;
use crate::physical_runtime::durability::{
    PhysicalRootPublicationIdentity, RetiredArtifact, RetirementRecord,
};
use crate::physical_runtime::record_serving::{
    planning::rebased_root::{
        plan_arena_forget, plan_retirement_release, RetirementRootPlanningContext,
    },
    RetirementCandidateRetryScope,
};
use worth_store_physical_format::RecordArtifactFile;

mod evidence;

impl RecordPublicationDirector {
    /// Recovery installs the durable obligation before ordinary publishers can
    /// acquire a lease. It does not turn replayed bytes into a write receipt.
    pub(in super::super) fn seed_extent_release(
        &self,
        records: &[RetirementRecord],
    ) -> Result<(), PhysicalRetirementDenial> {
        let mut records = records.iter().filter(|record| {
            matches!(
                record.artifact,
                RetiredArtifact::Extent { .. } | RetiredArtifact::Arena { .. }
            )
        });
        let Some(record) = records.next() else {
            return Ok(());
        };
        if records.next().is_some() {
            return Err(PhysicalRetirementDenial::Unresolved);
        }
        let release = record.release.ok_or(PhysicalRetirementDenial::WalPlan)?;
        let reserved = self
            .mutation_identity
            .reserve_mutation_identity()
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let operation = crate::physical_runtime::PhysicalMutationIdentity::from_reserved_operation(
            reserved.identity(),
        );
        let pending = self
            .root_owner
            .register_pending_publication(operation)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let displaced = DisplacedArtifact {
            source_root: record.source_root,
            artifact: record.artifact,
            bytes: record.bytes,
        };
        if !self.root_owner.claim_recovered_retirement(displaced) {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let intent = encode_retirement(
            record.artifact,
            false,
            record.source_root,
            record.bytes,
            Some(release),
        );
        if let RetiredArtifact::Arena { arena, .. } = record.artifact {
            self.preparation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .recovered_retiring_arena = Some(
                worth_store_physical_format::ExtentArenaId::new(arena)
                    .ok_or(PhysicalRetirementDenial::WalPlan)?,
            );
        }
        *self
            .retirement_release
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(PendingRetirementRelease {
            displaced,
            release,
            pending: Some(pending),
            intent,
            growth: None,
            growth_bytes: 0,
            progress: ReleaseProgress::Recovered(operation),
        });
        Ok(())
    }

    pub(super) fn resume_extent_release(
        &self,
        state: &mut PendingRetirementRelease,
        operation: crate::physical_runtime::PhysicalMutationIdentity,
    ) -> Result<(), PhysicalRetirementDenial> {
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
        if source.generation() >= state.release.candidate_generation() {
            self.verify_released_root(state, &allocation)?;
            // Bootstrap already loaded the published free index; the range may
            // even have been reused since. Never release it a second time.
            state.progress = ReleaseProgress::RootPublished {
                allocator_update: false,
            };
            return Ok(());
        }
        if source.generation() != state.release.source_generation() {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalRetirementDenial::Unresolved)?;
        let publication = state.release.publication();
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
        let (plan, free) = match state.displaced.artifact {
            RetiredArtifact::Extent { range, .. } => {
                plan_retirement_release(context, range, candidate)
            }
            RetiredArtifact::Arena { arena, .. } => plan_arena_forget(
                context,
                worth_store_physical_format::ExtentArenaId::new(arena)
                    .ok_or(PhysicalRetirementDenial::WalPlan)?,
                candidate,
            ),
            RetiredArtifact::Segment { .. } => return Err(PhysicalRetirementDenial::WalPlan),
        }
        .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        if plan.retained_metadata_bytes() != Some(state.release.metadata_bytes()) {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let retry = RetirementCandidateRetryScope::admit(state.release, source.generation(), &plan)
            .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let identity = PhysicalRootPublicationIdentity::from_retirement(
            self.durability.policy_identity(),
            operation,
            state.release,
            Sha256::digest(&state.intent).into(),
            publication,
        )
        .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let transition = self
            .root_owner
            .begin(identity, source.clone())
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        state.progress = ReleaseProgress::Prepared(ReleaseCandidate {
            source,
            free,
            plan,
            transition,
            allocation,
            retry: Some(retry),
        });
        Ok(())
    }
}
