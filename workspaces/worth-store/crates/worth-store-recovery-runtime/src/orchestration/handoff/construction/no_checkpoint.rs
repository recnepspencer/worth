//! Generation-zero construction demands the admitted absence witness.
use super::*;
impl RejoinConstruction {
    pub(super) fn no_checkpoint(
        self,
        core_body: PreparedRecoveredRuntimeCoreBacking,
        absent_checkpoint: Option<AbsentCheckpointWitness>,
        durability: Option<ConfiguredPhysicalDurabilityDeclaration>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Self {
            coordination,
            media,
            closed_cleanup,
        } = self;
        match (absent_checkpoint, durability) {
            (Some(absent), Some(declaration)) => {
                #[cfg(feature = "certification-test-authority")]
                if crate::certification::enabled() {
                    worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_generation_zero_with_backing_and_rejoin_pause(
                        core_body,
                        coordination,
                        media,
                        closed_cleanup,
                        declaration,
                        absent,
                        crate::certification::take_rejoin_pause(),
                    )
                } else {
                    worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_generation_zero_with_backing(
                        core_body, coordination, media, closed_cleanup, declaration, absent,
                    )
                }
                #[cfg(not(feature = "certification-test-authority"))]
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_generation_zero_with_backing(
                    core_body, coordination, media, closed_cleanup, declaration, absent,
                )
            }
            (Some(_absent), None) => Err(
                worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch,
            ),
            (None, _declaration) => worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_backing(
                core_body,
                coordination,
                media,
                closed_cleanup,
            ),
        }
    }
}
