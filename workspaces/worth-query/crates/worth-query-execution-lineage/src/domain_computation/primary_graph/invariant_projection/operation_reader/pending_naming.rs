//! Persistent-name reads of records an earlier member of the same edit
//! envelope created (#426): a later coupled producer or stage reads the name
//! and birth the envelope gives them, as commit finalization will record
//! them.
//!
//! The answers come from the envelope's own naming authority (the lent
//! coupled assembly's `ProposalNameSource`): its composed naming plan,
//! validated and minted over the applied state exactly as finalization
//! does, guarded against a naming basis that moved past the envelope's
//! basis. Nothing is reserved or written, so an envelope that refuses or
//! aborts leaves no trace on the branch's names.
//!
//! Such an answer seals. The record is created by this very commit, so its
//! minted name and birth are immutable once the commit seals, and the read
//! is keyed on the record's own fact (and, through it, on the member that
//! created it); an envelope that does not commit seals nothing. Every other
//! outcome still refuses typed and vetoes sealing:
//!
//! - a created record no member named refuses `UnpublishedCandidateRecord`;
//! - a refused composed plan or a stale naming basis refuses
//!   `CandidateNaming` with the exact naming denial;
//! - a name of another kind refuses with the read's own kind denial.
//!
//! A successor round reads what its own chain wrote as it was before the
//! chain (#277): the envelope's names describe the chain's current round,
//! so a record the chain covers, and a birth the envelope mints for one,
//! is never answered here.

use worth_query_declaration::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_execution_naming::facade::{CandidateBirth, ExecutionNameReadDenial};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::lineage::LineageRecordTarget;
use worth_relational::facade::runtime::{CustomInvariantNameReadDenial, CustomInvariantNameSource};

use super::naming::{PersistentNameReadVeto, WorthQueryDecisionNameDenial};
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead;

type Denial = WorthQueryDecisionNameDenial;
type PendingRead<T> = Result<Result<T, ExecutionNameReadDenial>, CustomInvariantNameReadDenial>;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// The name the envelope gives `record`, which the candidate created and
    /// has live as read. Refuses, vetoing sealing, when no member named it.
    pub(super) fn pending_name_of<K>(
        &mut self,
        record: LineageRecordTarget,
        read: impl FnOnce(&dyn CustomInvariantNameSource) -> PendingRead<WorthQueryName<K>>,
    ) -> Result<WorthQueryName<K>, Denial> {
        let unnamed = Denial::UnpublishedCandidateRecord(record);
        let answer = if self.own_chain_covers(record) {
            Err(unnamed)
        } else {
            match self.with_pending_names(read) {
                Ok(Some(Ok(name))) => Ok(name),
                Ok(None | Some(Err(ExecutionNameReadDenial::RecordNotNamed { .. }))) => {
                    Err(unnamed)
                }
                Ok(Some(Err(denial))) => Err(Denial::Read(denial)),
                Err(denial) => Err(denial),
            }
        };
        if answer.is_err() {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
        }
        answer
    }

    /// The birth the envelope mints for a held name the basis does not
    /// record: `None` when it mints none (or one for a record a successor's
    /// own chain covers), so the read refuses as an unborn name.
    pub(super) fn pending_birth<K>(
        &mut self,
        read: impl FnOnce(&dyn CustomInvariantNameSource) -> PendingRead<CandidateBirth<K>>,
    ) -> Result<Option<RecordedNameBirth<K>>, Denial> {
        let answer = match self.with_pending_names(read) {
            Ok(Some(Ok((birth, record)))) => {
                Ok((!record.is_some_and(|record| self.own_chain_covers(record))).then_some(birth))
            }
            Ok(None | Some(Err(ExecutionNameReadDenial::UnknownName))) => Ok(None),
            Ok(Some(Err(denial))) => Err(Denial::Read(denial)),
            Err(denial) => Err(denial),
        };
        if answer.is_err() {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
        }
        answer
    }

    /// Runs `read` over the envelope's names on an aligned applied
    /// candidate. `None` when this reader is lent no assembly whose current
    /// pin is its candidate: it then sees no member's names.
    fn with_pending_names<T>(
        &self,
        read: impl FnOnce(&dyn CustomInvariantNameSource) -> PendingRead<T>,
    ) -> Result<Option<Result<T, ExecutionNameReadDenial>>, Denial> {
        let NativeProjectionRead::Applied { candidate, .. } = &self.reader.native else {
            return Ok(None);
        };
        self.require_candidate_aligned()?;
        let Some(access) = self.source_query else {
            return Ok(None);
        };
        let mut read = Some(read);
        let mut answer = None;
        let lent = access.with_applied_candidate_names(
            self.reader.execution,
            candidate.candidate_identity(),
            &mut |names| answer = read.take().map(|read| read(names)),
        );
        match lent.and_then(|()| answer.transpose()) {
            Ok(answer) => Ok(answer),
            Err(CustomInvariantNameReadDenial::Unavailable) => Ok(None),
            Err(denial) => Err(Denial::CandidateNaming(denial)),
        }
    }

    /// Whether a successor round's own chain wrote `record`, so it is read
    /// as it was before the chain.
    fn own_chain_covers(&self, record: LineageRecordTarget) -> bool {
        let Some(own) = self.reader.native.own_outputs() else {
            return false;
        };
        match record {
            LineageRecordTarget::Entity(entity) => own.covers(entity),
            LineageRecordTarget::Relation(relation) => own.relations().covers(relation),
        }
    }
}
