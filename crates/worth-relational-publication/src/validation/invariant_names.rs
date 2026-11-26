//! The persistent names installed invariants read while one proposal is
//! validated.
//!
//! Committed reads answer from the branch's derived name index at the
//! proposal's basis commit. Proposed reads overlay the commit's own names,
//! previewed once, on first need, by the commit's naming authority
//! ([`preview_commit_names`]). A refused preview is kept: the owner refuses
//! the commit with that exact naming denial ([`ProposalNameSource::naming_denial`]).

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
    pending: OnceLock<Result<PendingCommitNames, LineageNameDenial>>,
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
}

impl<'a> ProposalNameSource<'a> {
    pub fn new(
        naming: &'a LineageNamingBasisBinding,
        schema: &'a RelationalSchemaRegistry,
        proposal: ProposalNameInputs<'a>,
    ) -> Self {
        Self {
            naming,
            schema,
            proposal,
            pending: OnceLock::new(),
        }
    }

    /// Settles a validation step that read these names: a naming denial a
    /// proposed read met refuses the commit ahead of that step's verdict,
    /// exactly as finalization would refuse it.
    pub fn settle<T>(
        self,
        step: Result<T, TransactionCommitError>,
    ) -> Result<T, TransactionCommitError> {
        match self.naming_denial() {
            Some(denial) => Err(TransactionCommitError::lineage_name_denial(denial)),
            None => step,
        }
    }

    /// The naming denial a proposed read met, which the commit refuses with.
    pub fn naming_denial(&self) -> Option<LineageNameDenial> {
        self.pending.get()?.as_ref().err().cloned()
    }

    fn pending(&self) -> Result<&PendingCommitNames, Denial> {
        let preview = self.pending.get_or_init(|| {
            let basis = self.naming.read_guard(self.proposal.branch.clone());
            let outcome = self.proposal.outcome;
            preview_commit_names(CommitNamePreviewInput {
                staged: self.proposal.staged,
                version_id: self.proposal.version_id,
                changed_records: &outcome.effect.publication.changed_records,
                created_entities: &outcome.created_entities,
                created_relations: &outcome.created_relations,
                naming_plan: self.proposal.naming_plan,
                scheme_registry: self.schema.persistent_name_schemes(),
                schema_registry: Some(self.schema),
                basis: &basis,
            })
        });
        preview
            .as_ref()
            .map_err(|denial| Denial::Naming(denial.clone()))
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
        match self.pending.and_then(|pending| pending.record_name(record)) {
            Some(PendingRecordName::Named(name)) => return RecordName::Named(name),
            Some(PendingRecordName::Ended) => return RecordName::Ended,
            None => {}
        }
        match self.committed.map(|view| view.name_of_record(record)) {
            Some(PersistentNameOfRecord::Live { name, .. }) => RecordName::Named(name),
            Some(PersistentNameOfRecord::Ended { .. }) => RecordName::Ended,
            Some(PersistentNameOfRecord::Unnamed) | None => RecordName::Unnamed,
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
        identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    ) -> ProposalNameSource<'a> {
        let proposal = ProposalNameInputs {
            branch: input.target_branch().clone(),
            basis: input.basis().observation().commit_id(),
            staged: self.state(),
            version_id: identity.proposed_version_id(),
            outcome: &self.outcome,
            naming_plan,
        };
        ProposalNameSource::new(lineage.naming, lineage.schema, proposal)
    }
}
