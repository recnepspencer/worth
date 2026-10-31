//! Refusals preserve the actual phase reached and its continuous Home lease.
use super::UnrecoveredApplicationHome;
use worth_relational::facade::runtime::RelationalRuntimeConstructionDenial;

pub enum RelationalAttachmentRefused {
    BeforeConstruction {
        cause: worth_execution::WorkCeilingDenial,
        home: worth_proof::HomeAdmitted<crate::AdmittedApplicationHome>,
    },
    Construction(RelationalAttachmentFailedHome),
}
impl std::fmt::Debug for RelationalAttachmentRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeConstruction { cause, .. } => {
                f.debug_tuple("BeforeConstruction").field(cause).finish()
            }
            Self::Construction(home) => f
                .debug_struct("Construction")
                .field("cause", &home.cause)
                .field("home", &home.home_identity())
                .finish(),
        }
    }
}
pub struct RelationalAttachmentFailedHome {
    pub(super) cause: RelationalRuntimeConstructionDenial,
    pub(super) context: crate::home_context::HomeContext,
}
impl RelationalAttachmentFailedHome {
    pub fn reason(&self) -> &RelationalRuntimeConstructionDenial {
        &self.cause
    }
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.context.descriptor.identity
    }
    /// No Native capture or checkpoint task was attempted. The original
    /// construction cause accompanies the existing physical close outcome.
    pub fn close(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> (
        RelationalRuntimeConstructionDenial,
        crate::ApplicationHomeCloseOutcome,
    ) {
        let Self { cause, context } = self;
        let custody = crate::ApplicationHomeRecoveryCustody::from_never_attached_home(context);
        (cause, custody.try_release(request))
    }
}

pub struct RelationalHomeReadmissionRefused {
    pub(super) cause: crate::ApplicationHomeOpenRefusal,
    pub(super) home: Box<UnrecoveredApplicationHome>,
}
impl std::fmt::Debug for RelationalHomeReadmissionRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalHomeReadmissionRefused")
            .field("cause", &self.cause)
            .field("home", &self.home.home_identity())
            .finish()
    }
}
impl RelationalHomeReadmissionRefused {
    pub fn reason(&self) -> &crate::ApplicationHomeOpenRefusal {
        &self.cause
    }
    pub fn into_home(self) -> UnrecoveredApplicationHome {
        *self.home
    }
}
