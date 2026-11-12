//! Live reactivation preserves originals rather than manufacturing recovered evidence.
use super::*;
pub(crate) struct ReactivatedCreationFacts {
    pub(crate) identity: crate::identity::ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) attempt: CompositePublicationAttemptIdentity,
    pub(crate) expected: ProductBranchObservation,
    pub(crate) admitted_at: RuntimeWorldInstant,
    pub(crate) deadline: Option<RuntimeWorldInstant>,
    pub(crate) progress: CompositeAttemptProgress,
    pub(crate) successor: Option<AdmittedCompositeRuntimeWorldBasis>,
    pub(crate) resources: ActiveAttemptResources,
    pub(crate) cause: crate::recovery::ProductUnpublishedCause,
    pub(crate) before_effect_refusal: Option<super::super::NoEffectCause>,
    pub(crate) last_observed: Option<crate::branch::ProductBranchReferenceSnapshot>,
    pub(crate) destination: Arc<crate::branch::registry::ProductBranchInstallationWitness>,
}
impl ActiveAttemptRecord {
    pub(crate) fn from_retained_creation(f: ReactivatedCreationFacts) -> Arc<Self> {
        Arc::new(Self {
            identity: f.identity,
            attempt: f.attempt,
            expected: f.expected,
            admitted_at: f.admitted_at,
            deadline: f.deadline,
            publication: None,
            state: std::sync::Mutex::new(super::record::ActiveAttemptState {
                progress: Arc::new(f.progress),
                successor: f.successor,
                resources: Some(f.resources),
                abandoned: false,
                cause: f.cause,
                before_effect_refusal: f.before_effect_refusal,
                last_observed: f.last_observed,
                destination: Some(f.destination),
            }),
        })
    }
}
impl ActiveAttemptCustody {
    pub(crate) fn from_reactivated_creation(
        record: Arc<ActiveAttemptRecord>,
        slot: ReservedProductUnpublishedSlot,
    ) -> Self {
        Self {
            record,
            slot: Some(slot),
        }
    }
}
