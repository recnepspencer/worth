//! Move the same concrete Home only after its persisted owner admission.
use super::WorldReadmittedApplicationHome;
use crate::open::readmission::transition::{
    readmit_no_pending_owners, ApplicationHomeOwnerAdmissionDenial,
};
use worth_execution::ExecutionRequest;

/// Store retains the actual Home, incarnation and lease through admission.
/// Descriptive data cannot construct this carrier.
///
/// ```compile_fail,E0451
/// use worth_store_application::{NoPendingApplicationHomeTransition, WorldReadmittedApplicationHome};
/// fn forge(home: WorldReadmittedApplicationHome<(), (), (), (), ()>) {
///     let _ = NoPendingApplicationHomeTransition { home };
/// }
/// ```
pub struct NoPendingApplicationHomeTransition {
    home: WorldReadmittedApplicationHome<(), (), (), (), ()>,
}

impl NoPendingApplicationHomeTransition {
    /// The closed Host completion consumes the actual admitted owner once.
    ///
    /// ```
    /// use worth_store_application::{NoPendingApplicationHomeTransition, WorldReadmittedApplicationHome};
    /// fn once(admitted: NoPendingApplicationHomeTransition) -> WorldReadmittedApplicationHome<(), (), (), (), ()> {
    ///     admitted.into_no_pending_home()
    /// }
    /// ```
    ///
    /// ```compile_fail,E0382
    /// use worth_store_application::NoPendingApplicationHomeTransition;
    /// fn twice(admitted: NoPendingApplicationHomeTransition) {
    ///     let _home = admitted.into_no_pending_home();
    ///     let _again = admitted.into_no_pending_home();
    /// }
    /// ```
    pub fn into_no_pending_home(self) -> WorldReadmittedApplicationHome<(), (), (), (), ()> {
        self.home
    }
}

pub struct ApplicationHomeOwnerAdmissionRefused {
    home: Box<WorldReadmittedApplicationHome<(), (), (), (), ()>>,
    cause: ApplicationHomeOwnerAdmissionDenial,
}
impl ApplicationHomeOwnerAdmissionRefused {
    pub fn reason(&self) -> &ApplicationHomeOwnerAdmissionDenial {
        &self.cause
    }
    pub fn into_home(self) -> WorldReadmittedApplicationHome<(), (), (), (), ()> {
        *self.home
    }
    pub fn into_parts(
        self,
    ) -> (
        WorldReadmittedApplicationHome<(), (), (), (), ()>,
        ApplicationHomeOwnerAdmissionDenial,
    ) {
        (*self.home, self.cause)
    }
}
impl std::fmt::Debug for ApplicationHomeOwnerAdmissionRefused {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationHomeOwnerAdmissionRefused")
            .field("home", &self.home.home_identity())
            .field("cause", &self.cause)
            .finish()
    }
}

impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    pub fn readmit_no_pending_owners(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<NoPendingApplicationHomeTransition, ApplicationHomeOwnerAdmissionRefused> {
        let checked = match self.home.relational.world_recovery_authority() {
            Some(recovered) => readmit_no_pending_owners(&self.home.context, recovered, request),
            None => Err(ApplicationHomeOwnerAdmissionDenial::NativeRecoveryRequired),
        };
        match checked {
            Ok(()) => Ok(NoPendingApplicationHomeTransition { home: self }),
            Err(cause) => Err(ApplicationHomeOwnerAdmissionRefused {
                home: Box::new(self),
                cause,
            }),
        }
    }
}
