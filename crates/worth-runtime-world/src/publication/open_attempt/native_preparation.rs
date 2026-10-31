//! One Native preparation joins the already committed original World open.
use super::WorldOpenedPublicationAttempt;
use crate::publication::{
    CompositePublicationStage, PreparedCompositePublicationWithoutSignal,
    ReservedCompositePublicationAttempt, WithoutSignal,
};
use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, RelationalCallerAssociation, ValidatedRelationalProposal,
};

#[derive(Debug)]
pub enum WorldNativePreparationDenial {
    Native(worth_relational::facade::transactions::TransactionCommitError),
    MissingAssociation,
    ForeignAssociation,
    Lowering(crate::publication::NoEffectCause),
    Durable(crate::durability::WorldDurableMediumDenial),
}

/// Refusal retains the same committed open, independently issued identities,
/// and capacities. It never presents the whole journey as no effect.
#[derive(Debug)]
pub struct WorldNativePreparationRefusal {
    open: WorldOpenedPublicationAttempt,
    cause: WorldNativePreparationDenial,
}
impl WorldNativePreparationRefusal {
    pub fn cause(&self) -> &WorldNativePreparationDenial {
        &self.cause
    }
    pub fn into_open(self) -> WorldOpenedPublicationAttempt {
        self.open
    }
}

/// Post-open preparation has its own cancellation outcome. The memory-only
/// prepared type's `cancel -> NoEffect` door is never exposed for this phase.
///
/// ```compile_fail,E0308
/// use worth_runtime_world::facade::{WorldPreparedPublicationWithoutSignal, NoEffectCompositePublication};
/// fn erase_open(prepared: WorldPreparedPublicationWithoutSignal) -> NoEffectCompositePublication {
///     prepared.cancel()
/// }
/// ```
///
/// ```
/// use worth_runtime_world::facade::{WorldPreparedPublicationWithoutSignal, ProductUnpublishedOwnerEffects};
/// fn retain_open(prepared: WorldPreparedPublicationWithoutSignal) -> ProductUnpublishedOwnerEffects {
///     prepared.cancel()
/// }
/// ```
#[derive(Debug)]
pub struct WorldPreparedPublicationWithoutSignal {
    pub(crate) prepared: PreparedCompositePublicationWithoutSignal,
}
impl WorldPreparedPublicationWithoutSignal {
    pub fn planned_successor(&self) -> &crate::publication::PlannedProductReferenceSuccessor {
        self.prepared.planned_successor()
    }
    /// Original issued identities, exposed only for descriptive correlation.
    pub fn attempt_identity(&self) -> &crate::identity::CompositePublicationAttemptIdentity {
        self.prepared.planned_successor().attempt_identity()
    }
    pub fn reserved_commit_identity(&self) -> crate::identity::CompositeCommitIdentity {
        self.prepared.reserved_commit_identity()
    }

    pub fn expected_head(&self) -> &crate::branch::ProductBranchObservation {
        self.prepared.expected_head()
    }
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.prepared.unpublished_recovery_handle()
    }
    pub fn cancel(self) -> crate::recovery::ProductUnpublishedOwnerEffects {
        self.prepared.into_attempt().retain_open()
    }
}

/// Linear custody of the SAME opened attempt after its one proposal binding.
/// The bound proposal is consumed by Native's existing preparation exactly once.
#[derive(Debug)]
pub struct WorldQueryNativePreparation {
    open: WorldOpenedPublicationAttempt,
}
impl WorldQueryNativePreparation {
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.open.unpublished_recovery_handle()
    }
    pub fn complete_query_native_preparation(
        self,
        candidate: PreparedRelationalCommitCandidate,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorldPreparedPublicationWithoutSignal, WorldNativePreparationRefusal> {
        self.open.complete_native_preparation(candidate, request)
    }
    pub fn retain(self) -> crate::recovery::ProductUnpublishedOwnerEffects {
        self.open.retain()
    }
}

impl WorldOpenedPublicationAttempt {
    /// Closed Query preparation consumes the actual open and original proposal.
    /// Candidate-dependent Query work then uses Native's one prepared candidate.
    pub fn bind_query_native_proposal(
        self,
        proposal: ValidatedRelationalProposal,
    ) -> Result<
        (WorldQueryNativePreparation, ValidatedRelationalProposal),
        WorldNativePreparationRefusal,
    > {
        match self.bind_native_proposal(proposal) {
            Ok(bound) => Ok((WorldQueryNativePreparation { open: self }, bound)),
            Err(cause) => Err(WorldNativePreparationRefusal {
                open: self,
                cause: WorldNativePreparationDenial::Native(cause),
            }),
        }
    }

    /// Consumes the one validated handler result before Native's existing
    /// preparation call. Association cannot be added to a sealed candidate.
    fn bind_native_proposal(
        &self,
        proposal: ValidatedRelationalProposal,
    ) -> Result<
        ValidatedRelationalProposal,
        worth_relational::facade::transactions::TransactionCommitError,
    > {
        let association = RelationalCallerAssociation::from_bytes(self.caller_association_bytes())
            .expect("World's bounded canonical association satisfies the Native opaque contract");
        proposal.bind_caller_association(association)
    }

    /// Seal one consumed, already-validated handler result using Native's
    /// existing preparation exactly once. No handler or proposal is replayed.
    pub fn prepare_native(
        self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        proposal: ValidatedRelationalProposal,
    ) -> Result<WorldPreparedPublicationWithoutSignal, WorldNativePreparationRefusal> {
        let proposal = match self.bind_native_proposal(proposal) {
            Ok(proposal) => proposal,
            Err(cause) => {
                return Err(WorldNativePreparationRefusal {
                    open: self,
                    cause: WorldNativePreparationDenial::Native(cause),
                })
            }
        };
        let candidate = match runtime.prepare_validated_proposal(request, proposal) {
            Ok(candidate) => candidate,
            Err(cause) => {
                return Err(WorldNativePreparationRefusal {
                    open: self,
                    cause: WorldNativePreparationDenial::Native(cause),
                })
            }
        };
        self.complete_native_preparation(candidate, request)
    }

    fn complete_native_preparation(
        mut self,
        candidate: PreparedRelationalCommitCandidate,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorldPreparedPublicationWithoutSignal, WorldNativePreparationRefusal> {
        let Some(association) = candidate.caller_association() else {
            return Err(WorldNativePreparationRefusal {
                open: self,
                cause: WorldNativePreparationDenial::MissingAssociation,
            });
        };
        let original = &self.original.record.association;
        let observed = crate::durability::decode_association(
            association.as_bytes(),
            original.attempt().owner().home(),
        );
        if observed.as_ref() != Ok(original) {
            return Err(WorldNativePreparationRefusal {
                open: self,
                cause: WorldNativePreparationDenial::ForeignAssociation,
            });
        }
        let plan = match crate::publication::lower_component_plans(
            self.original.resolved.clone(),
            Some(candidate),
            None,
            None,
        ) {
            Ok(plan) => plan,
            Err(cause) => {
                return Err(WorldNativePreparationRefusal {
                    open: self,
                    cause: WorldNativePreparationDenial::Lowering(cause.cause()),
                })
            }
        };
        if let Err(cause) = self
            .original
            .custody
            .prepare_durable_occurrence(&self.original.subject, request)
        {
            return Err(WorldNativePreparationRefusal {
                open: self,
                cause: WorldNativePreparationDenial::Durable(cause),
            });
        }
        let original = self.original;
        let expected = original.resolved.expected().clone();
        let attempt = ReservedCompositePublicationAttempt::from_open(
            original.identity,
            expected.clone(),
            expected.basis().clone(),
            plan,
            original.custody,
            original.deadline,
        );
        Ok(WorldPreparedPublicationWithoutSignal {
            prepared: WithoutSignal::seal(attempt),
        })
    }

    pub fn retain(self) -> crate::recovery::ProductUnpublishedOwnerEffects {
        self.original.custody.retain_open()
    }
}
