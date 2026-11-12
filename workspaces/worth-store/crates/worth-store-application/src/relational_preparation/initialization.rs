//! A physical new-Home opportunity is consumed before invoking the Host recipe.
use super::UnsharedReadmittedApplicationHome;
use worth_store::physical_runtime::RecoveryStorePresence;

pub(crate) enum HomeInitializationOrigin {
    Available(RecoveryStorePresence),
    ConsumedNewHome,
}
#[derive(Debug)]
pub enum ApplicationHomeInitializationDenial {
    Execution(worth_execution::WorkCeilingDenial),
    ExistingHomeRequiresQueryPublication,
    OpportunityAlreadyConsumed,
}
pub struct ApplicationHomeInitializationRefused {
    cause: ApplicationHomeInitializationDenial,
    home: Box<UnsharedReadmittedApplicationHome>,
}
impl std::fmt::Debug for ApplicationHomeInitializationRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationHomeInitializationRefused")
            .field("cause", &self.cause)
            .field("home", &self.home.home_identity())
            .finish()
    }
}
impl ApplicationHomeInitializationRefused {
    pub fn reason(&self) -> &ApplicationHomeInitializationDenial {
        &self.cause
    }
    pub fn into_home(self) -> UnsharedReadmittedApplicationHome {
        *self.home
    }
}
impl UnsharedReadmittedApplicationHome {
    pub fn initialize_new_home<R>(
        mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        body: impl FnOnce(Self) -> R,
    ) -> Result<R, ApplicationHomeInitializationRefused> {
        let denial = match request.consult() {
            Err(cause) => Some(ApplicationHomeInitializationDenial::Execution(cause)),
            Ok(()) => match self.context.initialization {
                HomeInitializationOrigin::Available(RecoveryStorePresence::ProvenAbsent) => None,
                HomeInitializationOrigin::Available(RecoveryStorePresence::ExistingArtifacts) => {
                    Some(ApplicationHomeInitializationDenial::ExistingHomeRequiresQueryPublication)
                }
                HomeInitializationOrigin::ConsumedNewHome => {
                    Some(ApplicationHomeInitializationDenial::OpportunityAlreadyConsumed)
                }
            },
        };
        if let Some(cause) = denial {
            return Err(ApplicationHomeInitializationRefused {
                cause,
                home: Box::new(self),
            });
        }
        // This records opportunity consumption, never callback completion.
        self.context.initialization = HomeInitializationOrigin::ConsumedNewHome;
        Ok(body(self))
    }
}
