//! Refusals retain the real predecessor, successor and physical opening phase.
use super::super::{refusal::WorthQueryApplicationHomeOpenRefused, runtime::DirectoryHome};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_home_integration::*, primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

#[derive(Debug)]
pub enum WorthQueryHomeProgramTransitionDenial {
    DirectoryRequired,
    BindingAdmission(worth_query_installation::facade::WorthQueryHomeBindingDenial),
    FiniteResourcesRequired,
    InvalidPlan,
    Author(WorthQueryPrimaryGraphInstallationDenial),
    AuthoredBody(WorthQueryHomeProgramTransitionBodyDenial),
    Native(worth_relational::facade::runtime::RelationalHomeProgramContractDenial),
    Body(worth_relational::facade::runtime::RelationalHomeProgramBodyDenial),
    Staging(worth_relational::facade::mvcc::RelationalTransactionStagingDenial),
    Commit(worth_relational::facade::transactions::TransactionCommitError),
    Indexes(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial),
    Intent(worth_store_application::ApplicationHomeTransitionIntentDenial),
    Plan(worth_store_application::ApplicationHomeTransitionPlanningDenial),
    IntentOutcome(worth_store_application::ApplicationHomeTransitionIntentOutcome),
    Open(worth_runtime_world::facade::WorldOpenAttemptOutcome),
    StoppedOpen {
        cause: worth_execution::WorkCeilingDenial,
        pending: worth_runtime_world::facade::WorldPendingOpenAttempt,
    },
    OpenPreparation {
        reservation: worth_runtime_world::facade::WorldHomeTransitionStepReservation,
        cause: worth_runtime_world::facade::WorldOpenAttemptRefusal,
    },
    NativePreparation(worth_runtime_world::facade::WorldNativePreparationRefusal),
    Publication(worth_runtime_world::facade::WorldDurablePublicationOutcome),
    StoppedPublication {
        cause: worth_execution::WorkCeilingDenial,
        pending: worth_runtime_world::facade::WorldPendingDurablePublication,
    },
    Completion(worth_store_application::ApplicationHomeTransitionStepRejected),
    Binding(worth_store_application::ApplicationHomeTransitionFinishDenial),
    Observation(
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::RuntimeWorldBranchAdmissionDenial,
        >,
    ),
    Composition(worth_store_application::ApplicationHomeQueryCompositionDenial),
    BridgeProgramme(Box<worth_runtime_bridge::facade::BridgeAuthoredProgrammeHandoffDenial>),
    Request(worth_execution::WorkCeilingDenial),
}

pub enum WorthQueryHomeProgramTransitionOpenPhase<
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
> {
    Predecessor {
        refused: WorthQueryApplicationHomeOpenRefused<A, AP>,
        successor: PreparedHomeApplication<B, BP>,
    },
    Successor(WorthQueryApplicationHomeOpenRefused<B, BP>),
    Configuration(WorthQueryHomeProgramTransitionDenial),
    BeforeCompilation {
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: PreparedHomeApplication<B, BP>,
        home: DirectoryHome,
        cause: WorthQueryHomeProgramTransitionDenial,
    },
    Compilation {
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: QueryHomeTransitionPreparationRefused<B, BP>,
        home: DirectoryHome,
    },
    Transition {
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: PreparedHomeProgramTransitionTarget<B, BP>,
        home: DirectoryHome,
        cause: WorthQueryHomeProgramTransitionDenial,
    },
    Promotion {
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: QueryHomeTransitionPromotionRefused<B, BP>,
        home: DirectoryHome,
    },
}
impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP> std::fmt::Debug
    for WorthQueryHomeProgramTransitionOpenPhase<A, AP, B, BP>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Predecessor { refused, .. } => {
                f.debug_tuple("Predecessor").field(refused).finish()
            }
            Self::Successor(refused) => f.debug_tuple("Successor").field(refused).finish(),
            Self::Configuration(cause) => f.debug_tuple("Configuration").field(cause).finish(),
            Self::BeforeCompilation { cause, .. } | Self::Transition { cause, .. } => {
                f.debug_tuple("Transition").field(cause).finish()
            }
            Self::Compilation { successor, .. } => f
                .debug_tuple("Compilation")
                .field(successor.reason())
                .finish(),
            Self::Promotion { successor, .. } => f
                .debug_tuple("Promotion")
                .field(successor.reason())
                .finish(),
        }
    }
}

/// Owning refusal keeps the exact phase on the heap, so error propagation does
/// not reserve another full Native/World custody value in every caller frame.
pub struct WorthQueryHomeProgramTransitionOpenRefused<
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
> {
    phase: Box<WorthQueryHomeProgramTransitionOpenPhase<A, AP, B, BP>>,
}
impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>
{
    pub fn phase(&self) -> &WorthQueryHomeProgramTransitionOpenPhase<A, AP, B, BP> {
        &self.phase
    }
    pub fn into_phase(self) -> Box<WorthQueryHomeProgramTransitionOpenPhase<A, AP, B, BP>> {
        self.phase
    }
    #[inline(never)]
    pub(super) fn predecessor(
        refused: WorthQueryApplicationHomeOpenRefused<A, AP>,
        successor: PreparedHomeApplication<B, BP>,
    ) -> Self {
        Self {
            phase: Box::new(WorthQueryHomeProgramTransitionOpenPhase::Predecessor {
                refused,
                successor,
            }),
        }
    }
    #[inline(never)]
    pub(super) fn successor(refused: WorthQueryApplicationHomeOpenRefused<B, BP>) -> Self {
        Self {
            phase: Box::new(WorthQueryHomeProgramTransitionOpenPhase::Successor(refused)),
        }
    }
    #[inline(never)]
    pub(super) fn configuration(cause: WorthQueryHomeProgramTransitionDenial) -> Self {
        Self {
            phase: Box::new(WorthQueryHomeProgramTransitionOpenPhase::Configuration(
                cause,
            )),
        }
    }
    #[inline(never)]
    pub(super) fn before_compilation(
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: PreparedHomeApplication<B, BP>,
        home: DirectoryHome,
        cause: WorthQueryHomeProgramTransitionDenial,
    ) -> Self {
        Self {
            phase: Box::new(
                WorthQueryHomeProgramTransitionOpenPhase::BeforeCompilation {
                    predecessor,
                    successor,
                    home,
                    cause,
                },
            ),
        }
    }
    #[inline(never)]
    pub(super) fn compilation(
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: QueryHomeTransitionPreparationRefused<B, BP>,
        home: DirectoryHome,
    ) -> Self {
        Self {
            phase: Box::new(WorthQueryHomeProgramTransitionOpenPhase::Compilation {
                predecessor,
                successor,
                home,
            }),
        }
    }
    #[inline(never)]
    pub(super) fn transition(
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: PreparedHomeProgramTransitionTarget<B, BP>,
        home: DirectoryHome,
        cause: WorthQueryHomeProgramTransitionDenial,
    ) -> Self {
        Self {
            phase: Box::new(WorthQueryHomeProgramTransitionOpenPhase::Transition {
                predecessor,
                successor,
                home,
                cause,
            }),
        }
    }
    #[inline(never)]
    pub(super) fn promotion(
        predecessor: PreparedExistingHomeSignalProgram<A, AP>,
        successor: QueryHomeTransitionPromotionRefused<B, BP>,
        home: DirectoryHome,
    ) -> Self {
        Self {
            phase: Box::new(WorthQueryHomeProgramTransitionOpenPhase::Promotion {
                predecessor,
                successor,
                home,
            }),
        }
    }
}
impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP> std::fmt::Debug
    for WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.phase.fmt(f)
    }
}
