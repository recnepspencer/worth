//! Tracked reads of recorded births on an operation's decision reader: a
//! held name's birth, and the name a birth of given parts was minted as.
//!
//! A birth visible at the reader's basis is immutable and stays visible at
//! every later basis, so an answer that found one rests on no fact. An
//! answer that found none rests on a birth not yet recorded, which no Query
//! fact key tracks: a committed decision then refuses to seal
//! ([`PersistentNameReadVeto::UntrackedDependency`]). On an applied
//! candidate a held name's birth an earlier envelope member mints answers
//! from the envelope's names (`pending_naming`, #426); any other answer that
//! found none refuses `CandidateNameHistory`, since the unpublished
//! candidate may yet bear that very birth.

use worth_query_declaration::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationEntityMarkerIdentity,
};
use worth_query_execution_naming::facade::{
    candidate_entity_birth_of, candidate_relation_birth_of, CandidateBirth,
    ExecutionNameReadDenial, InvariantNameRead, NameReadDependencies, WorthQueryOutputBirth,
};
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema, OperationReads};

use super::naming::{PersistentNameReadVeto, WorthQueryDecisionNameDenial};
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead;

type Denial = WorthQueryDecisionNameDenial;

/// One birth read, before an applied candidate's awaited birth is settled.
enum BirthRead<T> {
    Settled(Result<T, Denial>),
    /// The basis records no birth the answer needs; the candidate may.
    Awaited {
        refusal: Denial,
        veto: PersistentNameReadVeto,
    },
}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// The recorded birth of a held entity name at this reader's basis, as
    /// a tracked decision read: decode it with `RecordedNameBirth::decode`.
    /// A split, merged or retired name still has its birth.
    pub fn birth_of<Entity>(
        &mut self,
        held: &WorthQueryName<Entity>,
    ) -> Result<RecordedNameBirth<Entity>, Denial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.admit_name_read(Entity::IDENTIFIER)?;
        let answer = self.tracked_birth_read(|reader, dependencies| {
            reader.entity_birth_of::<Schema, Entity>(held, dependencies)
        });
        self.or_pending_birth(answer, |names| {
            candidate_entity_birth_of::<Schema, Entity>(names, held)
        })
    }

    /// The recorded birth of a held relation name at this reader's basis.
    pub fn relation_birth_of<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        held: &WorthQueryName<Relation>,
    ) -> Result<RecordedNameBirth<Relation>, Denial>
    where
        Relation: fragment::Relation + OperationReads<Operation>,
    {
        self.admit_relation_name_read(relation.name())?;
        let answer = self.tracked_birth_read(|reader, dependencies| {
            reader.relation_birth_of::<Relation>(held, dependencies)
        });
        self.or_pending_birth(answer, |names| {
            candidate_relation_birth_of::<Relation>(names, held)
        })
    }

    /// The entity name a birth of exactly these parts was minted as, when
    /// that birth is recorded and visible at this reader's basis; `None`
    /// otherwise. Resolve the name to find where its meaning lives now.
    pub fn name_for_birth<Entity>(
        &mut self,
        birth: &WorthQueryOutputBirth,
        parents: &[WorthQueryName<Entity>],
    ) -> Result<Option<WorthQueryName<Entity>>, Denial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.admit_name_read(Entity::IDENTIFIER)?;
        self.settled_birth_read(|reader, dependencies| {
            reader.entity_name_for_birth::<Schema, Entity>(birth, parents, dependencies)
        })
    }

    /// The relation name a birth of exactly these parts was minted as.
    pub fn relation_name_for_birth<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        birth: &WorthQueryOutputBirth,
        parents: &[WorthQueryName<Relation>],
    ) -> Result<Option<WorthQueryName<Relation>>, Denial>
    where
        Relation: fragment::Relation + OperationReads<Operation>,
    {
        self.admit_relation_name_read(relation.name())?;
        self.settled_birth_read(|reader, dependencies| {
            reader.relation_name_for_birth::<Relation>(birth, parents, dependencies)
        })
    }

    /// A birth read's answer, or, on an applied candidate whose basis does
    /// not record the held name, the birth an earlier envelope member mints
    /// for it (`pending_birth`). A pending birth replaces the awaited-birth
    /// refusal and its veto: it rests on this commit's own immutable birth.
    fn or_pending_birth<K>(
        &mut self,
        answer: BirthRead<RecordedNameBirth<K>>,
        pending: impl FnOnce(
            &dyn worth_relational::facade::runtime::CustomInvariantNameSource,
        ) -> InvariantNameRead<CandidateBirth<K>>,
    ) -> Result<RecordedNameBirth<K>, Denial> {
        match answer {
            BirthRead::Settled(answer) => answer,
            BirthRead::Awaited { refusal, veto } => match self.pending_birth(pending)? {
                Some(birth) => Ok(birth),
                None => {
                    self.decision_facts.veto_persistent_name(veto);
                    Err(refusal)
                }
            },
        }
    }

    /// Runs one birth read and settles what it rested on: an awaited birth
    /// refuses on an applied candidate and vetoes sealing otherwise; a
    /// refused read vetoes sealing.
    fn settled_birth_read<T>(
        &mut self,
        read: impl FnOnce(
            &worth_query_execution_naming::facade::ExecutionNameReader<'_>,
            &mut NameReadDependencies,
        ) -> Result<T, ExecutionNameReadDenial>,
    ) -> Result<T, Denial> {
        match self.tracked_birth_read(read) {
            BirthRead::Settled(answer) => answer,
            BirthRead::Awaited { refusal, veto } => {
                self.decision_facts.veto_persistent_name(veto);
                Err(refusal)
            }
        }
    }

    /// One birth read, settled except for an answer that awaits a birth the
    /// basis does not record, whose refusal and veto are left to the caller.
    fn tracked_birth_read<T>(
        &mut self,
        read: impl FnOnce(
            &worth_query_execution_naming::facade::ExecutionNameReader<'_>,
            &mut NameReadDependencies,
        ) -> Result<T, ExecutionNameReadDenial>,
    ) -> BirthRead<T> {
        let mut dependencies = NameReadDependencies::new();
        let read = self
            .with_name_reader(|reader| (read(reader, &mut dependencies), reader.branch().clone()));
        let (answer, branch) = match read {
            Ok(read) => read,
            Err(denial) => return BirthRead::Settled(Err(denial)),
        };
        // On an applied candidate an answer awaiting only a birth the basis
        // does not record may be borne by the unpublished candidate itself.
        if matches!(self.reader.native, NativeProjectionRead::Applied { .. })
            && dependencies.awaits_births_on(&branch)
            && dependencies.ended_lineages_on(&branch).is_empty()
            && dependencies.live_records_on(&branch).is_empty()
        {
            return BirthRead::Awaited {
                refusal: Denial::CandidateNameHistory,
                veto: PersistentNameReadVeto::ReadFailed,
            };
        }
        if let Err(denial) = self.require_candidate_footprint(&dependencies, &branch) {
            return BirthRead::Settled(Err(denial));
        }
        let veto = match &answer {
            Err(ExecutionNameReadDenial::UnknownName) => {
                Some(PersistentNameReadVeto::UntrackedDependency)
            }
            Err(_) => Some(PersistentNameReadVeto::ReadFailed),
            Ok(_) if dependencies.awaits_births_on(&branch) => {
                Some(PersistentNameReadVeto::UntrackedDependency)
            }
            Ok(_) => None,
        };
        if let Some(veto) = veto {
            self.decision_facts.veto_persistent_name(veto);
        }
        BirthRead::Settled(answer.map_err(Denial::Read))
    }
}
