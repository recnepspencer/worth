//! A naming scope: one authored occurrence bound once for every birth and
//! posture a candidate declares under it.
//!
//! The scope is a thin facade over this candidate's own naming methods. It
//! holds no state besides the occurrence: every call declares through the
//! candidate writer exactly what the matching manual call declares, so a
//! scoped and a manual author mint byte-identical names. Roles are typed
//! (`WorthQueryNameRole`), so no role name is spelled at a birth site.
use worth_query_declaration::facade::application_naming::{
    ApplicationGroupKind, ApplicationNameSubjectKind, OccurrencePath, OccurrencePathDenial,
    WorthQueryGroupPartition, WorthQueryName, WorthQueryNameRole, WorthQuerySameCommitName,
};
use worth_query_execution_naming::facade::{
    WorthQueryLeastMergeKeeper, WorthQueryOutputBirth, WorthQueryOutputMergeKeeper,
};
use worth_query_installation::facade::{ApplicationEntityRef, OperationCreates};

use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityKey;

mod relations;

/// Births and postures of one authored occurrence, borrowed from a
/// candidate writer (`CandidateWriter::naming_scope`).
pub struct NamingScope<'scope, 'borrow, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    writer: &'scope mut CandidateWriter<'borrow, Schema, Binding>,
    occurrence: OccurrencePath,
}

impl<'borrow, Schema, Binding> CandidateWriter<'borrow, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Opens a naming scope binding `occurrence` (the authored identity of
    /// one feature, instance or assembly member) for every birth under it.
    pub fn naming_scope(
        &mut self,
        occurrence: OccurrencePath,
    ) -> NamingScope<'_, 'borrow, Schema, Binding> {
        NamingScope {
            writer: self,
            occurrence,
        }
    }
}

impl<'borrow, Schema, Binding> NamingScope<'_, 'borrow, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub fn occurrence(&self) -> &OccurrencePath {
        &self.occurrence
    }

    /// The writer, for effects (field writes, deletes, links) between
    /// declarations.
    pub fn writer(&mut self) -> &mut CandidateWriter<'borrow, Schema, Binding> {
        self.writer
    }

    /// A child occurrence (`assembly/window`) of this one, reborrowing the
    /// same writer.
    pub fn sub_scope(
        &mut self,
        segment: &str,
    ) -> Result<NamingScope<'_, 'borrow, Schema, Binding>, OccurrencePathDenial> {
        let occurrence = self.occurrence.clone().child(segment)?;
        Ok(self.writer.naming_scope(occurrence))
    }

    /// An indexed child occurrence (`pattern/instance[2]`) of this one;
    /// indices order numerically.
    pub fn index(
        &mut self,
        name: &str,
        index: u32,
    ) -> Result<NamingScope<'_, 'borrow, Schema, Binding>, OccurrencePathDenial> {
        let occurrence = self.occurrence.clone().index(name, index)?;
        Ok(self.writer.naming_scope(occurrence))
    }

    /// A birth in this occurrence under `role`; chain
    /// `with_discriminator` to tell siblings of one role apart.
    pub fn birth<R: WorthQueryNameRole>(&self, role: R) -> WorthQueryOutputBirth {
        WorthQueryOutputBirth::of_role(&self.occurrence, role)
    }

    /// Creates an entity in the partition the commit issues and names it.
    pub fn create_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
        birth: WorthQueryOutputBirth,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Binding::Operation>,
    {
        let created = self.writer.create_entity(entity, key)?;
        self.writer.name_created_entity(&created, birth)?;
        Ok(created)
    }

    /// Creates an entity in `context`'s partition and names it.
    pub fn create_entity_in_context<Entity, ContextEntity>(
        &mut self,
        context: &WorthQueryApplicationEffectEntity<Schema, ContextEntity>,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
        birth: WorthQueryOutputBirth,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Binding::Operation>,
    {
        let created = self.writer.create_entity_in_context(context, entity, key)?;
        self.writer.name_created_entity(&created, birth)?;
        Ok(created)
    }

    /// Names an entity this candidate created.
    pub fn name_entity<Entity>(
        &mut self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.name_created_entity(created, birth)
    }

    /// Regenerates `held` in place: its record, deleted by this candidate,
    /// ends, and `created` is born under the same name (`Preserve`) when
    /// `birth` is the birth `held` was minted from.
    pub fn preserve_entity<Entity>(
        &mut self,
        held: WorthQueryName<Entity>,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.retire_entity_name(held)?;
        self.writer.name_created_entity(created, birth)
    }

    /// Splits `parent`, deleted by this candidate, into created `children`.
    pub fn split_entity<'c, Entity: 'c>(
        &mut self,
        parent: WorthQueryName<Entity>,
        children: impl IntoIterator<
            Item = (
                &'c WorthQueryApplicationEffectEntity<Schema, Entity>,
                WorthQueryOutputBirth,
            ),
        >,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Schema: 'c,
    {
        self.writer
            .split_entity_name(parent, children.into_iter().collect())
    }

    /// Ends `held` with the entity this candidate deletes.
    pub fn retire_entity<Entity>(
        &mut self,
        held: WorthQueryName<Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.retire_entity_name(held)
    }

    /// Merges held entity names under an explicitly chosen `keeper`.
    pub fn merge_entity<Entity>(
        &mut self,
        parents: &[WorthQueryName<Entity>],
        keeper: WorthQueryOutputMergeKeeper<
            Entity,
            &WorthQueryApplicationEffectEntity<Schema, Entity>,
        >,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.merge_entity_names(parents, keeper)
    }

    /// Merges held entity names keeping the least of them (canonical
    /// `WorthQueryName` order), whatever order `parents` lists them in.
    pub fn merge_least_entity<Entity>(
        &mut self,
        parents: &[WorthQueryName<Entity>],
        keeper: WorthQueryLeastMergeKeeper<&WorthQueryApplicationEffectEntity<Schema, Entity>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let (ordered, keeper) = keeper
            .resolve(ApplicationNameSubjectKind::Entity, parents)
            .map_err(WorthQueryApplicationAttemptDenial::persistent_name_output)?;
        self.writer.merge_entity_names(&ordered, keeper)
    }

    /// Declares group kind `G`'s next partition from this scope; members
    /// may be outputs this scope just named (`same_commit_entity_name`).
    pub fn declare_groups<G: ApplicationGroupKind>(
        &mut self,
        partition: WorthQueryGroupPartition<G>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.writer.declare_group_partition(partition)
    }

    /// The same-commit reference to `created`'s name, for a later-role
    /// discriminator (`NameRef::from` it).
    pub fn same_commit_entity_name<Entity>(
        &self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<WorthQuerySameCommitName<Entity>, WorthQueryApplicationAttemptDenial> {
        self.writer.same_commit_entity_name(created)
    }
}
