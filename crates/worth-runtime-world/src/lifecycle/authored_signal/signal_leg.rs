//! Prepare a recovered mixed original's planned Signal leg through the real
//! Bridge projection owner, at the original's recorded Signal basis.
use super::*;
use crate::publication::durable_signal::{WorldDurableSignalAdvance, WorldDurableSignalBacking};
use worth_foundational::facade::CompositeAttemptAssociation;
use worth_runtime_bridge::facade::{
    BridgeDurableDefinitionPublicationDenial, BridgePlannedDefinitionLeg,
    BridgePreparedAuthoredDefinitionPublication, BridgeReadmittedAuthoredAssembly,
};
use worth_signal::facade::durability::SignalCompositeAdvanceBinding;

/// Why a recovered planned Signal leg could not begin. Every refusal
/// precedes any Signal effect.
#[derive(Debug)]
pub enum WorldSignalLegDenial {
    /// Only a Bridge projection owner prepares an authored definition leg.
    BridgeUnavailable,
    /// The current program does not prepare the recorded leg.
    Bridge(Box<BridgeDurableDefinitionPublicationDenial>),
    /// Signal no longer stands at the original's recorded basis.
    SourceMoved,
    Signal(Box<SignalDurableAdvanceDenial>),
    Capacity,
}

/// The one recovered leg, bound to its original association and basis.
pub(crate) struct PreparedWorldSignalLeg {
    original: CompositeAttemptAssociation,
    assembly: Arc<BridgeReadmittedAuthoredAssembly>,
    definition: BridgePreparedAuthoredDefinitionPublication,
    binding: SignalCompositeAdvanceBinding,
    backing: WorldDurableSignalBacking,
}
impl PreparedWorldSignalLeg {
    /// The readmitted durable open and planned leg stand in for the lost live
    /// attempt. Only readmission mints a recovered record, from those rows.
    pub(crate) fn begin_recovered(
        self,
        request: ExecutionRequest<'_, '_>,
        original: &crate::recovery::RecoveredWorldPublicationRecord,
        issuer: &InstalledCompositeSignalPublicationOwner,
        cancellation: &worth_signal::facade::branch::SignalOwnerCancellationToken,
    ) -> Result<Arc<WorldDurableSignalAdvance>, WorldSignalLegDenial> {
        assert_eq!(
            original.open.association, self.original,
            "the recovered original binds its own Signal leg"
        );
        let grant = issuer.admit_original(self.binding.commitment());
        let mutation = self.assembly.signal_owner().mutation_port();
        let task = self
            .definition
            .begin_composite(
                &mutation,
                request,
                self.binding,
                grant,
                &mut (),
                cancellation,
                |_| Ok(()),
            )
            .map_err(|cause| WorldSignalLegDenial::Bridge(Box::new(cause)))?;
        Ok(self.backing.retain(task))
    }
}

impl<D, I, E, Ctx, T> RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn prepare_planned_signal_leg(
        &self,
        request: &ExecutionRequest<'_, '_>,
        planned: &BridgePlannedDefinitionLeg,
        expected: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        association: CompositeAttemptAssociation,
    ) -> Result<PreparedWorldSignalLeg, WorldSignalLegDenial> {
        let WorldAuthoredSignalProjectionOwner::Bridge(owner) = &self.projection else {
            return Err(WorldSignalLegDenial::BridgeUnavailable);
        };
        let assembly = owner
            .upgrade()
            .ok_or(WorldSignalLegDenial::BridgeUnavailable)?;
        let definition = assembly
            .prepare_planned_definition_publication(request, planned)
            .map_err(|cause| WorldSignalLegDenial::Bridge(Box::new(cause)))?;
        if !definition.admits_signal_basis(expected) {
            return Err(WorldSignalLegDenial::SourceMoved);
        }
        let backing =
            WorldDurableSignalBacking::reserve(request).map_err(|_| WorldSignalLegDenial::Capacity)?;
        let binding = assembly
            .signal_owner()
            .mutation_port()
            .prepare_composite_binding(request, expected, association.clone())
            .map_err(|cause| WorldSignalLegDenial::Signal(Box::new(cause)))?;
        Ok(PreparedWorldSignalLeg {
            original: association,
            assembly,
            definition,
            binding,
            backing,
        })
    }
}
