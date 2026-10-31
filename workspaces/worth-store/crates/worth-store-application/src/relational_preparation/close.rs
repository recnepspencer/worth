//! Pre-service teardown owns raw Native; it never shares or checkpoints it.
use super::{UnrecoveredApplicationHome, UnsharedReadmittedApplicationHome};
use crate::{ApplicationHomeCloseDenial as Denial, ApplicationHomeCloseOutcome as Outcome};
use worth_execution::ExecutionRequest;

pub struct UnrecoveredApplicationHomeCloseRefused {
    cause: Denial,
    home: UnrecoveredApplicationHome,
}
pub struct UnsharedApplicationHomeCloseRefused {
    cause: Denial,
    home: UnsharedReadmittedApplicationHome,
}
impl std::fmt::Debug for UnrecoveredApplicationHomeCloseRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnrecoveredApplicationHomeCloseRefused")
            .field("cause", &self.cause)
            .field("home", &self.home.home_identity())
            .finish()
    }
}
impl std::fmt::Debug for UnsharedApplicationHomeCloseRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnsharedApplicationHomeCloseRefused")
            .field("cause", &self.cause)
            .field("home", &self.home.home_identity())
            .finish()
    }
}

impl UnrecoveredApplicationHomeCloseRefused {
    pub fn reason(&self) -> &Denial {
        &self.cause
    }
    pub fn into_home(self) -> UnrecoveredApplicationHome {
        self.home
    }
    pub fn try_close(self, request: &ExecutionRequest<'_, '_>) -> Result<Outcome, Self> {
        self.home.close(request)
    }
}
impl UnsharedApplicationHomeCloseRefused {
    pub fn reason(&self) -> &Denial {
        &self.cause
    }
    pub fn into_home(self) -> UnsharedReadmittedApplicationHome {
        self.home
    }
    pub fn try_close(self, request: &ExecutionRequest<'_, '_>) -> Result<Outcome, Self> {
        self.home.close(request)
    }
}

impl UnrecoveredApplicationHome {
    pub fn close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Outcome, UnrecoveredApplicationHomeCloseRefused> {
        if let Err(cause) = request.consult() {
            return Err(UnrecoveredApplicationHomeCloseRefused {
                cause: Denial::Execution(cause),
                home: self,
            });
        }
        let Self {
            relational,
            context,
        } = self;
        let closing = match relational.prepare_close() {
            Ok(closing) => closing,
            Err((relational, cause)) => {
                return Err(UnrecoveredApplicationHomeCloseRefused {
                    cause: Denial::Relational(cause),
                    home: Self {
                        relational,
                        context,
                    },
                })
            }
        };
        Ok(
            crate::ApplicationHomeRecoveryCustody::from_pre_service_home(context, closing)
                .try_release(request),
        )
    }
}
impl UnsharedReadmittedApplicationHome {
    pub fn close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Outcome, UnsharedApplicationHomeCloseRefused> {
        if let Err(cause) = request.consult() {
            return Err(UnsharedApplicationHomeCloseRefused {
                cause: Denial::Execution(cause),
                home: self,
            });
        }
        let Self {
            relational,
            context,
        } = self;
        let closing = match relational.prepare_close() {
            Ok(closing) => closing,
            Err((relational, cause)) => {
                return Err(UnsharedApplicationHomeCloseRefused {
                    cause: Denial::Relational(cause),
                    home: Self {
                        relational,
                        context,
                    },
                })
            }
        };
        Ok(
            crate::ApplicationHomeRecoveryCustody::from_pre_service_home(context, closing)
                .try_release(request),
        )
    }
}
