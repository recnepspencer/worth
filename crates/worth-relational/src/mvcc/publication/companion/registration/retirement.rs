//! Detach the exact callback registration without service admission or payload Drop.
use super::{
    registry_guard, CompanionRegistrationState, PublicationCompanionRegistration,
    PublicationCompanionRegistrationPort, PublicationCompanionRegistrationStop,
};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationCompanionRetirementDenial {
    Registration(PublicationCompanionRegistrationStop),
    Execution(WorkCeilingDenial),
}

/// Inert original custody after callback routing was removed. Final destruction
/// may run the participant's Drop and is outside the controlled retirement step.
/// This exposes no service, participant, registration or close authority.
#[derive(Debug)]
pub struct RetiredPublicationCompanionRegistration {
    _removed: CompanionRegistrationState,
    _registration: PublicationCompanionRegistration,
}

impl RetiredPublicationCompanionRegistration {
    /// Reclaim only outside a request-controlled operation.
    pub fn reclaim(self) {
        drop(self);
    }
}

impl PublicationCompanionRegistrationPort {
    /// Retire this exact registration without waiting or invoking its participant.
    /// Refusal returns the original registration. Required posture and generation
    /// survive; unrelated or superseding registrations are never removed.
    /// Cleanup remains legal after service admission stops.
    pub fn try_retire_required_for_request(
        &self,
        registration: PublicationCompanionRegistration,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        RetiredPublicationCompanionRegistration,
        (
            PublicationCompanionRegistration,
            PublicationCompanionRetirementDenial,
        ),
    > {
        let removed = (|| {
            let mut state = registry_guard::acquire(
                self.publication.companion_registry().state.try_write(),
                PublicationCompanionRegistrationStop::PublicationPending,
            )
            .map_err(PublicationCompanionRetirementDenial::Registration)?;
            self.require_local(&registration)
                .map_err(PublicationCompanionRetirementDenial::Registration)?;
            let generation = match &*state {
                CompanionRegistrationState::RequiredActive { generation, .. }
                    if *generation == registration.generation =>
                {
                    *generation
                }
                _ => {
                    return Err(PublicationCompanionRetirementDenial::Registration(
                        PublicationCompanionRegistrationStop::Superseded,
                    ));
                }
            };
            request
                .consult()
                .map_err(PublicationCompanionRetirementDenial::Execution)?;
            // No fallible work or arbitrary destruction follows this point.
            Ok(std::mem::replace(
                &mut *state,
                CompanionRegistrationState::RequiredRebind { generation },
            ))
        })();
        match removed {
            Ok(removed) => Ok(RetiredPublicationCompanionRegistration {
                _removed: removed,
                _registration: registration,
            }),
            Err(reason) => Err((registration, reason)),
        }
    }
}
