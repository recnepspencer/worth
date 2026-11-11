//! Replace the programme of the one actual authored assembly before Query service.
use super::*;
use std::sync::Arc;
use worth_runtime_bridge::facade::{
    BridgeConditionalRuntimeBuilder, BridgeReadmittedAuthoredAssembly,
};
#[derive(Debug)]
pub enum ApplicationHomeProgrammeHandoffDenial {
    CompositionNotAttached,
    AssemblyRetained,
    Bridge(Box<worth_runtime_bridge::facade::BridgeAuthoredProgrammeHandoffDenial>),
}
pub struct ApplicationHomeProgrammeHandoffRefused {
    builder: Box<BridgeConditionalRuntimeBuilder>,
    cause: ApplicationHomeProgrammeHandoffDenial,
}
impl ApplicationHomeProgrammeHandoffRefused {
    pub fn reason(&self) -> &ApplicationHomeProgrammeHandoffDenial {
        &self.cause
    }
    pub fn into_parts(
        self,
    ) -> (
        Box<BridgeConditionalRuntimeBuilder>,
        ApplicationHomeProgrammeHandoffDenial,
    ) {
        (self.builder, self.cause)
    }
}
impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    pub fn handoff_query_programme(
        &mut self,
        builder: Box<BridgeConditionalRuntimeBuilder>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<BridgeReadmittedAuthoredAssembly>, ApplicationHomeProgrammeHandoffRefused> {
        use ApplicationHomeProgrammeHandoffDenial as D;
        let Some(composition) = &mut self.composition else {
            return Err(ApplicationHomeProgrammeHandoffRefused {
                builder,
                cause: D::CompositionNotAttached,
            });
        };
        let Some(original) = composition.conditional.take() else {
            return Err(ApplicationHomeProgrammeHandoffRefused {
                builder,
                cause: D::CompositionNotAttached,
            });
        };
        let original = match Arc::try_unwrap(original) {
            Ok(original) => Box::new(original),
            Err(original) => {
                composition.conditional = Some(original);
                return Err(ApplicationHomeProgrammeHandoffRefused {
                    builder,
                    cause: D::AssemblyRetained,
                });
            }
        };
        match original.rebind_relational_programme(builder, request) {
            Ok(original) => {
                let original: Arc<BridgeReadmittedAuthoredAssembly> = Arc::from(original);
                composition.conditional = Some(Arc::clone(&original));
                Ok(original)
            }
            Err((original, builder, cause)) => {
                composition.conditional = Some(Arc::from(original));
                Err(ApplicationHomeProgrammeHandoffRefused {
                    builder,
                    cause: D::Bridge(Box::new(cause)),
                })
            }
        }
    }
}
