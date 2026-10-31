//! Failure keeps the same lease, Native section, World slot and actual source.
pub struct WorldHomeReadmissionRefused {
    pub(super) cause: crate::ApplicationHomeOpenRefusal,
    pub(super) home: Box<crate::ReadmittedApplicationHome>,
}

impl std::fmt::Debug for WorldHomeReadmissionRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldHomeReadmissionRefused")
            .field("cause", &self.cause)
            .field("home", &self.home.home_identity())
            .field("incarnation", &self.home.incarnation())
            .finish_non_exhaustive()
    }
}

impl WorldHomeReadmissionRefused {
    pub fn reason(&self) -> &crate::ApplicationHomeOpenRefusal {
        &self.cause
    }

    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.home.home_identity()
    }

    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.home.incarnation()
    }

    /// Explicit cleanup of the failed admission returns the SAME concrete
    /// Native-readmitted Home; it grants no reconciled World stage.
    pub fn into_home(self) -> crate::ReadmittedApplicationHome {
        let Self { home, cause } = self;
        drop(cause);
        *home
    }
}
