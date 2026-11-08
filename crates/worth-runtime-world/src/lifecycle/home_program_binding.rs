//! Before-service acknowledgement changes meaning on the same World medium.
use crate::durability::{WorldDurableHomeBinding, WorldDurableMediumBinding};
use worth_proof::{
    HomeProgramTransitionAuthority, HomeProgramTransitionBindingCommitted,
    HomeProgramTransitionRequired,
};

#[derive(Debug)]
pub enum WorldHomeProgramBindingDenial {
    OwnerUnavailable,
    ForeignOwner,
    StaleIncarnation,
    NotHomeBound,
    AlreadyBound,
    Busy,
    SourceChanged,
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    Readmission(crate::durability::WorldReadmissionDenial),
}

/// Issued only inside the actual final product-source guard. It owns no port
/// and cannot escape the callback or be used to supply a different revision.
pub struct PreparedWorldHomeProgramBinding<'owner> {
    medium: &'owner WorldDurableMediumBinding,
    required: &'owner HomeProgramTransitionRequired,
    authority: &'owner HomeProgramTransitionAuthority,
    successor: WorldDurableHomeBinding,
    _reservation: crate::durability::HomeProgramBindingReservation<'owner>,
}
impl<'owner> PreparedWorldHomeProgramBinding<'owner> {
    pub(in crate::lifecycle) fn prepare(
        medium: &'owner WorldDurableMediumBinding,
        expected: &crate::branch::ProductBranchObservation,
        required: &'owner HomeProgramTransitionRequired,
        successor: &worth_proof::AdmittedHomeProgramTransitionSuccessor<
            &worth_foundational::facade::ApplicationHomeBinding,
        >,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldHomeProgramBindingDenial> {
        use WorldHomeProgramBindingDenial as Denial;
        let binding = medium.binding();
        let authority = binding.transition_authority().ok_or(Denial::NotHomeBound)?;
        authority
            .admit_successor(required, successor)
            .map_err(Denial::Admission)?;
        let revision = *successor.binding().selected().revision().bytes();
        let attempt = expected
            .publication_attempt()
            .ok_or(Denial::SourceChanged)?;
        let owner = attempt
            .owner_identity()
            .durable()
            .ok_or(Denial::NotHomeBound)?;
        if owner != binding.owner() {
            return Err(Denial::ForeignOwner);
        }
        let through =
            worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(
                owner,
                attempt.ordinal(),
            );
        let successor = binding
            .successor(revision, through, request)
            .map_err(Denial::Readmission)?;
        let reservation = medium.reserve_home_program_binding()?;
        Ok(Self {
            medium,
            required,
            authority,
            successor,
            _reservation: reservation,
        })
    }

    pub fn acknowledge(
        self,
        acknowledgement: &HomeProgramTransitionBindingCommitted,
    ) -> Result<(), WorldHomeProgramBindingDenial> {
        self.authority
            .admit_binding_committed(self.required, acknowledgement)
            .map_err(WorldHomeProgramBindingDenial::Admission)?;
        self.medium.install_home_binding(self.successor);
        Ok(())
    }
}
