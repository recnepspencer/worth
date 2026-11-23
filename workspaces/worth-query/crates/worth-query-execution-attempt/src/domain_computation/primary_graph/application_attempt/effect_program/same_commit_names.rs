//! References to the names this program's own named outputs will be born
//! with, for the discriminators of its later-role outputs.
//!
//! A reference is issued only for an output this program created and has
//! already named, so a referencing birth always follows the birth it
//! references. It carries no digest: the commit writes the referenced name
//! into the discriminator once Relational's minting inputs are all known.

use worth_query_declaration::facade::application_naming::{
    ApplicationNameSubjectKind as Subject, WorthQuerySameCommitName,
};
use worth_query_execution_lineage::facade::persistent_naming::{
    same_commit_name_of, OperationOutputKey, WorthQueryOutputNameDenial,
};

use super::persistent_names::refused;
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCreatedRelation,
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    /// The reference to the name `created`, an entity this program created
    /// and already named, will be born with. Refuses an entity this program
    /// did not create or has not named.
    pub fn same_commit_entity_name<Entity>(
        &self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<WorthQuerySameCommitName<Entity>, WorthQueryApplicationAttemptDenial> {
        let (target, entity) = self.created_entity_key(created)?;
        self.same_commit_name(Subject::Entity, entity, &target)
    }

    /// The reference to the name `created`, a relation this program linked
    /// and already named, will be born with.
    pub fn same_commit_relation_name<Relation, From, To>(
        &self,
        created: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
    ) -> Result<WorthQuerySameCommitName<Relation>, WorthQueryApplicationAttemptDenial> {
        let (target, relation) = self.created_relation_key(created)?;
        self.same_commit_name(Subject::Relation, relation, &target)
    }

    fn same_commit_name<K>(
        &self,
        subject: Subject,
        subject_name: String,
        target: &OperationOutputKey,
    ) -> Result<WorthQuerySameCommitName<K>, WorthQueryApplicationAttemptDenial> {
        same_commit_name_of(&self.persistent_names, target).ok_or_else(|| {
            refused(WorthQueryOutputNameDenial::UnnamedOutputReference {
                subject,
                subject_name,
            })
        })
    }
}
