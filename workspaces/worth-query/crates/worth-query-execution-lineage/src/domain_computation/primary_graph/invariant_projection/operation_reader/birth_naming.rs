//! Tracked reads of recorded births on an operation's decision reader: a
//! held name's birth, and the name a birth of given parts was minted as.
//!
//! A birth visible at the reader's basis is immutable and stays visible at
//! every later basis, so an answer that found one rests on no fact. An
//! answer that found none rests on a birth not yet recorded, which no Query
//! fact key tracks: a committed decision then refuses to seal
//! ([`PersistentNameReadVeto::UntrackedDependency`]), and on an applied
//! candidate the read refuses `CandidateNameHistory`, since the unpublished
//! candidate may bear that very birth.

use worth_query_declaration::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationEntityMarkerIdentity,
};
use worth_query_execution_naming::facade::{
    ExecutionNameReadDenial, NameReadDependencies, WorthQueryOutputBirth,
};
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema, OperationReads};

use super::naming::{PersistentNameReadVeto, WorthQueryDecisionNameDenial};
use super::WorthQueryApplicationOperationInvariantProjectionReader;

type Denial = WorthQueryDecisionNameDenial;

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
        self.tracked_birth_read(|reader, dependencies| {
            reader.entity_birth_of::<Schema, Entity>(held, dependencies)
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
        self.tracked_birth_read(|reader, dependencies| {
            reader.relation_birth_of::<Relation>(held, dependencies)
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
        self.tracked_birth_read(|reader, dependencies| {
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
        self.tracked_birth_read(|reader, dependencies| {
            reader.relation_name_for_birth::<Relation>(birth, parents, dependencies)
        })
    }

    /// Runs one birth read and settles what it rested on: an awaited birth
    /// refuses on an applied candidate and vetoes sealing otherwise; a
    /// refused read vetoes sealing.
    fn tracked_birth_read<T>(
        &mut self,
        read: impl FnOnce(
            &worth_query_execution_naming::facade::ExecutionNameReader<'_>,
            &mut NameReadDependencies,
        ) -> Result<T, ExecutionNameReadDenial>,
    ) -> Result<T, Denial> {
        let mut dependencies = NameReadDependencies::new();
        let (answer, branch) = self.with_name_reader(|reader| {
            (read(reader, &mut dependencies), reader.branch().clone())
        })?;
        self.require_candidate_footprint(&dependencies, &branch)?;
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
        answer.map_err(Denial::Read)
    }
}
