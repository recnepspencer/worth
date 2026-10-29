//! Live Source custody stays with the proposal/materialization owner, not its receipts.
use crate::branch::{AdmittedRelationalBranchBasis, RelationalMaterializationRecord};
use crate::identity::data::VersionId;
use crate::mvcc::RelationalMaterializationTransactionMode;
use std::sync::{Arc, Weak};

/// Minted from the real suspension read. Retains its actual basis and manifest;
/// neither this backing nor its identifiers grant an installed Source audience.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NativeMaterializationSource {
    basis: AdmittedRelationalBranchBasis,
    records: Arc<Vec<RelationalMaterializationRecord>>,
}

impl NativeMaterializationSource {
    pub(crate) fn from_suspension(
        basis: &AdmittedRelationalBranchBasis,
        records: Arc<Vec<RelationalMaterializationRecord>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            basis: basis.clone(),
            records,
        })
    }

    pub(crate) fn basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }

    pub(crate) fn records(&self) -> &[RelationalMaterializationRecord] {
        &self.records
    }
}

/// One proposal's owner-issued interpretation. It is not a projected-row loan,
/// an installed read audience, a resumption correspondence or World authority.
#[derive(Debug)]
pub(crate) struct NativeManagedSourceInterpretation {
    publication: crate::runtime::RelationalRuntimePublicationBinding,
    physical_basis: AdmittedRelationalBranchBasis,
    proposed_version: VersionId,
    posture: Option<RelationalMaterializationTransactionMode>,
    materialization: Option<Arc<NativeMaterializationSource>>,
    pub(super) prepared: std::sync::OnceLock<super::staged::PreparedManagedIndexSources>,
}

impl NativeManagedSourceInterpretation {
    pub(crate) fn issue(
        input: &crate::mvcc::RelationalTransactionValidationInput,
        proposed_version: VersionId,
        publication: crate::runtime::RelationalRuntimePublicationBinding,
    ) -> Arc<Self> {
        Arc::new(Self {
            publication,
            physical_basis: input.basis().clone(),
            proposed_version,
            posture: input.intent().materialization_mode(),
            materialization: input.intent().materialization_source().cloned(),
            prepared: std::sync::OnceLock::new(),
        })
    }

    pub(crate) fn identity(self: &Arc<Self>) -> NativeSourceInterpretationIdentity {
        NativeSourceInterpretationIdentity(Arc::downgrade(self))
    }

    pub(crate) fn suspension_source_version(&self) -> Option<VersionId> {
        matches!(
            self.posture,
            Some(RelationalMaterializationTransactionMode::Suspend)
        )
        .then(|| self.physical_basis.observation().version_id())
    }

    pub(crate) fn physical_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.physical_basis
    }

    pub(crate) const fn proposed_version(&self) -> VersionId {
        self.proposed_version
    }

    pub(crate) const fn materialization_mode(
        &self,
    ) -> Option<RelationalMaterializationTransactionMode> {
        self.posture
    }

    /// Internal replay has a physical mode but no live port custody. It cannot
    /// obtain a managed Source loan/correspondence from that mode alone.
    pub(crate) fn materialization_source(&self) -> Option<&NativeMaterializationSource> {
        self.materialization.as_deref()
    }

    pub(super) fn require_current_registration(
        &self,
    ) -> Result<(), super::ManagedIndexSourcePreparationDenial> {
        use super::ManagedIndexSourcePreparationDenial as Denial;
        let prepared = self.prepared.get().ok_or(Denial::IncompletePreparation)?;
        if prepared.programs.is_empty() {
            return Ok(());
        }
        let epoch = self
            .publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        if generation != prepared.registration_generation {
            return Err(Denial::Publication(
                crate::mvcc::CompanionPreflightStop::RegistrationChanged,
            ));
        }
        Ok(())
    }
}

/// Receipts compare the actual owner identity without keeping its roots alive.
/// Upgrading this weak link is not Source admission; installed audience and the
/// exact actual observation still have to be checked by the receiving owner.
#[derive(Clone, Debug)]
pub(crate) struct NativeSourceInterpretationIdentity(Weak<NativeManagedSourceInterpretation>);

impl NativeSourceInterpretationIdentity {
    pub(crate) fn owner(&self) -> Option<Arc<NativeManagedSourceInterpretation>> {
        self.0.upgrade()
    }
}

impl PartialEq for NativeSourceInterpretationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for NativeSourceInterpretationIdentity {}
