//! Relation births and postures of a naming scope.
use worth_query_declaration::facade::application_naming::{
    ApplicationNameSubjectKind, WorthQueryName, WorthQuerySameCommitName,
};
use worth_query_execution_naming::facade::{
    WorthQueryLeastMergeKeeper, WorthQueryOutputBirth, WorthQueryOutputMergeKeeper,
};
use worth_query_installation::facade::OperationLinks;

use super::super::*;
use super::NamingScope;
use crate::domain_computation::primary_graph::WorthQueryApplicationCreatedRelation;

type Created<Schema, Relation, From, To> =
    WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>;

impl<Schema, Binding> NamingScope<'_, '_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Links `from` to `to` and names the new relation in one call.
    pub fn link_named<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
        birth: WorthQueryOutputBirth,
    ) -> Result<Created<Schema, Relation, From, To>, WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Binding::Operation>,
    {
        let created = self
            .writer
            .link_for_initialization(relation, key, from, to)?;
        self.writer.name_created_relation(&created, birth)?;
        Ok(created)
    }

    /// Names a relation this candidate linked for initialization.
    pub fn name_relation<Relation, From, To>(
        &mut self,
        created: &Created<Schema, Relation, From, To>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.name_created_relation(created, birth)
    }

    /// Regenerates `held` in place onto the created relation (`Preserve`).
    pub fn preserve_relation<Relation, From, To>(
        &mut self,
        held: WorthQueryName<Relation>,
        created: &Created<Schema, Relation, From, To>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.retire_relation_name(held)?;
        self.writer.name_created_relation(created, birth)
    }

    /// Splits relation `parent`, unlinked by this candidate, into created
    /// relation `children`.
    pub fn split_relation<'c, Relation: 'c, From: 'c, To: 'c>(
        &mut self,
        parent: WorthQueryName<Relation>,
        children: impl IntoIterator<
            Item = (
                &'c Created<Schema, Relation, From, To>,
                WorthQueryOutputBirth,
            ),
        >,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Schema: 'c,
    {
        self.writer
            .split_relation_name(parent, children.into_iter().collect())
    }

    /// Ends `held` with the relation this candidate unlinks.
    pub fn retire_relation<Relation>(
        &mut self,
        held: WorthQueryName<Relation>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.retire_relation_name(held)
    }

    /// Merges held relation names under an explicitly chosen `keeper`.
    pub fn merge_relation<Relation, From, To>(
        &mut self,
        parents: &[WorthQueryName<Relation>],
        keeper: WorthQueryOutputMergeKeeper<Relation, &Created<Schema, Relation, From, To>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.merge_relation_names(parents, keeper)
    }

    /// Merges held relation names keeping the least of them, whatever
    /// order `parents` lists them in.
    pub fn merge_least_relation<Relation, From, To>(
        &mut self,
        parents: &[WorthQueryName<Relation>],
        keeper: WorthQueryLeastMergeKeeper<&Created<Schema, Relation, From, To>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let (ordered, keeper) = keeper
            .resolve(ApplicationNameSubjectKind::Relation, parents)
            .map_err(WorthQueryApplicationAttemptDenial::persistent_name_output)?;
        self.writer.merge_relation_names(&ordered, keeper)
    }

    /// The same-commit reference to `created`'s name.
    pub fn same_commit_relation_name<Relation, From, To>(
        &self,
        created: &Created<Schema, Relation, From, To>,
    ) -> Result<WorthQuerySameCommitName<Relation>, WorthQueryApplicationAttemptDenial> {
        self.writer.same_commit_relation_name(created)
    }
}
