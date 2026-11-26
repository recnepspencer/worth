//! The persistent names installed invariants read while one proposal is
//! validated.
//!
//! Committed reads answer from the branch's derived name index at the
//! proposal's basis commit. Proposed reads overlay the commit's own names,
//! previewed once, on first need, by the commit's naming authority
//! ([`preview_commit_names`]), only while the branch's naming basis is still
//! the proposal's basis. A refused or stale preview is kept: the owner
//! refuses the commit with that exact naming denial, or with a retryable
//! stale-basis conflict ([`ProposalNameSource::settle`]).

use std::sync::OnceLock;

use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port as lineage;
use worth_relational_mutation::internal_port::validation::custom_rule::{
    CustomInvariantNamePosture as Posture, CustomInvariantNameReadDenial as Denial,
    CustomInvariantNameSource, CustomInvariantNameView, CustomInvariantRecordName as RecordName,
};

use contracts::history::data::coordinates::{BranchId, CommitId};
use contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameOfRecord};
use contracts::lineage::data::{
    LineageNameDenial, LineageRecordTarget, PersistentNameBirthParts, PersistentNameSubjectKind,
};
use contracts::schema::data::RelationalSchemaRegistry;
use contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use lineage::lineage::access::name_observation_view::{
    PersistentNameObservationView, PersistentNameObservationViewOwnerPort as ViewPort,
};
use lineage::lineage::authority::{
    preview_commit_names, CommitNamePreviewInput, PendingCommitNames, PendingRecordName,
};
use lineage::runtime::state::subsystems::lineage::naming_basis_binding::LineageNamingBasisBinding;

/// One proposal's persistent names for its installed invariants.
pub struct ProposalNameSource<'a> {
    naming: &'a LineageNamingBasisBinding,
    schema: &'a RelationalSchemaRegistry,
    proposal: ProposalNameInputs<'a>,
    pending: OnceLock<NamePreview>,
}

/// What one proposal's name preview settled to.
#[derive(Debug)]
pub enum NamePreview {
    Names(PendingCommitNames),
    Refused(LineageNameDenial),
    /// The branch's naming basis holds a later commit's lineage than the
    /// proposal's basis: a preview there would read another commit's names,
    /// so it never runs.
    Stale {
        basis: Option<CommitId>,
        latest: CommitId,
    },
}

/// A settled preview, carried into a later step of the same proposal whose
/// applied state is unchanged, so its names are not minted twice.
#[derive(Debug, Default)]
pub struct ProposalNameCarry(Option<NamePreview>);

impl ProposalNameCarry {
    /// This carry when `unchanged`; otherwise nothing.
    pub fn kept_while(self, unchanged: bool) -> Self {
        if unchanged {
            self
        } else {
            Self(None)
        }
    }
}

/// The applied proposal, its branch and basis commit.
pub struct ProposalNameInputs<'a> {
    pub branch: BranchId,
    /// `None` for an empty branch, which has no committed names.
    pub basis: Option<CommitId>,
    pub staged: &'a worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    pub version_id: contracts::identity::data::primitives::VersionId,
    pub outcome: &'a worth_relational_mutation::internal_port::authority::mutation::execution::MutationApplyOutcome,
    pub naming_plan: Option<&'a contracts::lineage::data::CommitNamingPlan>,
    pub graph_selection: Option<&'a worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection>,
}

impl<'a> ProposalNameSource<'a> {
    pub fn new(
        naming: &'a LineageNamingBasisBinding,
        schema: &'a RelationalSchemaRegistry,
        proposal: ProposalNameInputs<'a>,
        carry: ProposalNameCarry,
    ) -> Self {
        let pending = OnceLock::new();
        if let Some(preview) = carry.0 {
            let _ = pending.set(preview);
        }
        Self {
            naming,
            schema,
            proposal,
            pending,
        }
    }

    /// Settles a validation step that read these names (`settle_step`) and
    /// returns the preview to carry into the proposal's next step.
    pub fn settle<T>(
        self,
        step: Result<T, TransactionCommitError>,
    ) -> (Result<T, TransactionCommitError>, ProposalNameCarry) {
        let preview = self.pending.into_inner();
        (
            settle_step(preview.as_ref(), step),
            ProposalNameCarry(preview),
        )
    }

    fn pending(&self) -> Result<&PendingCommitNames, Denial> {
        let preview = self.pending.get_or_init(|| {
            let basis = self.naming.read_guard(self.proposal.branch.clone());
            if let Some(latest) = basis.latest_event_commit()
                && !naming_basis_current(Some(latest), self.proposal.basis)
            {
                let basis = self.proposal.basis;
                return NamePreview::Stale { basis, latest };
            }
            let outcome = self.proposal.outcome;
            let preview = preview_commit_names(CommitNamePreviewInput {
                staged: self.proposal.staged,
                version_id: self.proposal.version_id,
                changed_records: &outcome.effect.publication.changed_records,
                created_entities: &outcome.created_entities,
                created_relations: &outcome.created_relations,
                naming_plan: self.proposal.naming_plan,
                scheme_registry: self.schema.persistent_name_schemes(),
                schema_registry: Some(self.schema),
                basis: &basis,
                graph_selection: self.proposal.graph_selection,
            });
            preview.map_or_else(NamePreview::Refused, NamePreview::Names)
        });
        match preview {
            NamePreview::Names(names) => Ok(names),
            NamePreview::Refused(denial) => Err(Denial::Naming(denial.clone())),
            NamePreview::Stale { .. } => Err(Denial::StaleBasis),
        }
    }
}

/// Whether a branch naming basis whose latest lineage commit is `latest`
/// is still the basis of a proposal at `basis`: nothing after it.
pub fn naming_basis_current(latest: Option<CommitId>, basis: Option<CommitId>) -> bool {
    latest.is_none_or(|latest| basis.is_some_and(|basis| latest <= basis))
}

/// A step that read the proposal's names, settled against what its preview
/// met. A cancellation or execution stop stays the step's own. Otherwise a
/// refused naming plan refuses with its exact naming denial, and a stale
/// basis with a retryable stale-basis conflict, ahead of any verdict that
/// read rests on.
pub fn settle_step<T>(
    preview: Option<&NamePreview>,
    step: Result<T, TransactionCommitError>,
) -> Result<T, TransactionCommitError> {
    use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
    use contracts::transactions::data::outcomes::{commit_conflict, conflict_class};
    if matches!(
        step,
        Err(TransactionCommitError::Interrupted { .. } | TransactionCommitError::Execution { .. })
    ) {
        return step;
    }
    match preview {
        Some(NamePreview::Refused(denial)) => {
            Err(TransactionCommitError::lineage_name_denial(denial.clone()))
        }
        Some(NamePreview::Stale { basis, latest }) => Err(TransactionCommitError::conflict(
            commit_conflict::CommitConflict::new(
                conflict_class::ConflictClass::StaleValidationBasis {
                    detail: format!(
                        "branch names advanced to commit {latest:?} past proposal basis {basis:?}"
                    ),
                },
            ),
        )),
        Some(NamePreview::Names(_)) | None => step,
    }
}

impl CustomInvariantNameSource for ProposalNameSource<'_> {
    fn read(
        &self,
        posture: Posture,
        read: &mut dyn FnMut(&dyn CustomInvariantNameView),
    ) -> Result<(), Denial> {
        let pending = match posture {
            Posture::Committed => None,
            Posture::Proposed => Some(self.pending()?),
        };
        // The preview's own guard is released before this one is taken.
        let guard = self.naming.read_guard(self.proposal.branch.clone());
        let index = guard
            .name_index()
            .map_err(|denial| Denial::IndexUnavailable(format!("{denial:?}")))?;
        let committed = self
            .proposal
            .basis
            .map(|basis| <PersistentNameObservationView<'_> as ViewPort<'_>>::new(index, basis));
        read(&ProposalNames { committed, pending });
        Ok(())
    }

    fn scheme_subject(&self, scheme_id: &str) -> Option<(PersistentNameSubjectKind, String)> {
        let scheme = self.schema.persistent_name_scheme(scheme_id)?;
        Some((scheme.subject_kind(), scheme.subject_name().to_owned()))
    }
}

/// Committed names at the basis, overlaid with the commit's own when
/// proposed. An empty branch has no committed names.
struct ProposalNames<'v> {
    committed: Option<PersistentNameObservationView<'v>>,
    pending: Option<&'v PendingCommitNames>,
}

impl CustomInvariantNameView for ProposalNames<'_> {
    fn record_name(&self, record: LineageRecordTarget) -> RecordName {
        let pending = self.pending.and_then(|pending| pending.record_name(record));
        if let Some(PendingRecordName::Named(name)) = pending {
            return RecordName::Named(name);
        }
        let committed = match self.committed.map(|view| view.name_of_record(record)) {
            Some(PersistentNameOfRecord::Live { name, .. }) => RecordName::Named(name),
            Some(PersistentNameOfRecord::Ended { .. }) => RecordName::Ended,
            Some(PersistentNameOfRecord::Unnamed) | None => RecordName::Unnamed,
        };
        // A record the commit retires ends the name it carried; one it never
        // carried a name stays unnamed.
        match (pending, committed) {
            (Some(PendingRecordName::Ended), RecordName::Named(_)) => RecordName::Ended,
            (_, committed) => committed,
        }
    }

    fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; 32],
    ) -> Option<PersistentNameKey> {
        self.pending
            .and_then(|pending| pending.recorded_name(subject, digest))
            .or_else(|| self.committed?.recorded_name(subject, digest))
    }

    fn scheme_of(&self, name: PersistentNameKey) -> Option<&str> {
        match self.pending.and_then(|pending| pending.birth_of(name)) {
            Some(birth) => Some(birth.parts().scheme_id()),
            None => self.committed.as_ref()?.scheme_of(name),
        }
    }

    fn birth_of(&self, name: PersistentNameKey) -> Option<&PersistentNameBirthParts> {
        match self.pending.and_then(|pending| pending.birth_of(name)) {
            Some(birth) => Some(birth.parts()),
            None => self.committed.as_ref()?.birth_of(name),
        }
    }
}

impl crate::authority::commit::phases::applied_mutation::AppliedDraftCustody {
    /// This applied proposal's persistent names as its installed invariants
    /// read them, over the same naming basis and schema finalization uses.
    pub fn proposal_names<'a>(
        &'a self,
        lineage: crate::authority::commit::lineage_preparation::LineagePreparationAuthority<'a>,
        input: &'a impl crate::authority::commit::pipeline::publication_validation::DraftValidationInput,
        naming_plan: Option<&'a contracts::lineage::data::CommitNamingPlan>,
        plan: worth_relational_mutation::internal_port::authority::commit::semantic_plan::meaning::RelationalSemanticPlan<'a>,
        identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        carry: ProposalNameCarry,
    ) -> ProposalNameSource<'a> {
        let proposal = ProposalNameInputs {
            branch: input.target_branch().clone(),
            basis: input.basis().observation().commit_id(),
            staged: self.state(),
            version_id: identity.proposed_version_id(),
            outcome: &self.outcome,
            naming_plan,
            graph_selection: plan.graph_selection(),
        };
        ProposalNameSource::new(lineage.naming, lineage.schema, proposal, carry)
    }
}

#[cfg(test)]
#[path = "invariant_names_tests.rs"]
mod tests;
