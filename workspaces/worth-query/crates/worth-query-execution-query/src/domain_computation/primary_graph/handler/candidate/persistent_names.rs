//! Handler-facing persistent-name births and postures over this candidate's
//! outputs. Each call only declares meaning; Relational mints and validates
//! every name against the performed effects at commit.
use worth_query_declaration::facade::application_naming::{
    WorthQueryName, WorthQuerySameCommitName,
};
use worth_query_execution_naming::facade::{WorthQueryOutputBirth, WorthQueryOutputMergeKeeper};

use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCreatedRelation;

impl<Schema, Binding> CandidateWriter<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Declares the birth of an entity this candidate created. Two
    /// independently authored occurrences never share a name, even with
    /// identical payloads.
    pub fn name_created_entity<Entity>(
        &mut self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.name_created_entity(created, birth)
    }

    /// Declares the birth of a relation this candidate linked through
    /// `link_for_initialization`.
    pub fn name_created_relation<Relation, From, To>(
        &mut self,
        created: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.name_created_relation(created, birth)
    }

    /// The reference to the name an entity this candidate created and
    /// already named will be born with. A later-role output of the same
    /// occurrence embeds it in its discriminator; nothing else accepts it.
    pub fn same_commit_entity_name<Entity>(
        &self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<WorthQuerySameCommitName<Entity>, WorthQueryApplicationAttemptDenial> {
        self.candidate.same_commit_entity_name(created)
    }

    /// The reference to the name a relation this candidate linked and
    /// already named will be born with.
    pub fn same_commit_relation_name<Relation, From, To>(
        &self,
        created: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
    ) -> Result<WorthQuerySameCommitName<Relation>, WorthQueryApplicationAttemptDenial> {
        self.candidate.same_commit_relation_name(created)
    }

    /// Ends `held` with the entity this candidate deletes, so the same
    /// candidate may re-mint it (`Preserve`).
    pub fn retire_entity_name<Entity>(
        &mut self,
        held: WorthQueryName<Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.retire_entity_name(held)
    }

    /// Ends `held` with the relation this candidate unlinks.
    pub fn retire_relation_name<Relation>(
        &mut self,
        held: WorthQueryName<Relation>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.retire_relation_name(held)
    }

    /// Splits `parent`, deleted by this candidate, into created `children`.
    pub fn split_entity_name<Entity>(
        &mut self,
        parent: WorthQueryName<Entity>,
        children: Vec<(
            &WorthQueryApplicationEffectEntity<Schema, Entity>,
            WorthQueryOutputBirth,
        )>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.split_entity_name(parent, children)
    }

    /// Splits relation `parent`, unlinked by this candidate.
    #[allow(clippy::type_complexity)]
    pub fn split_relation_name<Relation, From, To>(
        &mut self,
        parent: WorthQueryName<Relation>,
        children: Vec<(
            &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
            WorthQueryOutputBirth,
        )>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.split_relation_name(parent, children)
    }

    /// Merges held entity names under `keeper`.
    pub fn merge_entity_names<Entity>(
        &mut self,
        parents: &[WorthQueryName<Entity>],
        keeper: WorthQueryOutputMergeKeeper<
            Entity,
            &WorthQueryApplicationEffectEntity<Schema, Entity>,
        >,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.merge_entity_names(parents, keeper)
    }

    /// Merges held relation names under `keeper`.
    pub fn merge_relation_names<Relation, From, To>(
        &mut self,
        parents: &[WorthQueryName<Relation>],
        keeper: WorthQueryOutputMergeKeeper<
            Relation,
            &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        >,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.merge_relation_names(parents, keeper)
    }
}
