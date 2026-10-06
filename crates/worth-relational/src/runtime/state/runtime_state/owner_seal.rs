use super::{RelationalRuntime, RelationalRuntimeTenure};

/// How a non-blocking in-place seal of one runtime owner finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRuntimeSealOutcome {
    /// This call stopped admission for good and resolved publication.
    Sealed,
    /// An earlier seal already did; nothing changed.
    AlreadySealed,
}

/// Why a non-blocking in-place seal was refused. Nothing changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRuntimeSealDenial {
    /// An admitted operation was still in flight, so the seal did not wait for
    /// it. Admission was never stopped.
    AdmissionsActive,
    /// This handle is one admitted operation borrowing the owner's state. It
    /// carries no owner authority, so it cannot seal the runtime it borrowed.
    NotOwner,
}

impl RelationalRuntime {
    /// Seal this runtime owner in place without waiting for admitted work.
    ///
    /// If any admitted operation is in flight the call is refused as
    /// [`RelationalRuntimeSealDenial::AdmissionsActive`] and admission was
    /// never stopped. Otherwise admission stops for good, publication
    /// settlement is resolved, and the runtime stays sealed: every service that
    /// admits through this owner denies as owner-unavailable and observes the
    /// owner as closed, even while another handle keeps the state alive, and
    /// dropping the owner closes nothing again. Snapshots taken earlier are
    /// frozen reads and keep reading.
    pub fn try_seal(
        &mut self,
    ) -> Result<RelationalRuntimeSealOutcome, RelationalRuntimeSealDenial> {
        let seal = match &self.tenure {
            RelationalRuntimeTenure::Owner(close) => close.try_seal()?,
            RelationalRuntimeTenure::Sealed(_) => {
                return Ok(RelationalRuntimeSealOutcome::AlreadySealed)
            }
            RelationalRuntimeTenure::Admitted(_) => {
                return Err(RelationalRuntimeSealDenial::NotOwner)
            }
        };
        let spent = std::mem::replace(&mut self.tenure, RelationalRuntimeTenure::Sealed(seal));
        if let RelationalRuntimeTenure::Owner(close) = spent {
            close.resolve_sealed_publication();
        }
        Ok(RelationalRuntimeSealOutcome::Sealed)
    }
}
