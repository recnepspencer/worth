//! An attempt's declared output names, bound to the exact created references
//! its sealed batch lowered and then lowered against the commit basis into
//! the one naming plan its Relational transaction carries.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind;
use worth_query_execution_lineage::facade::persistent_naming::{
    AuthoredOperationNaming, BoundOperationNaming, OperationOutputKey, WorthQueryOutputNameDenial,
};
use worth_relational::facade::lineage::{CommitNamingPlan, PendingBirthTarget};
use worth_relational::facade::transactions::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, MutationIntent,
};

use super::{WorthQueryApplicationAttemptDenial, WorthQueryRegisteredProviderEffects};

/// Naming posture of one registered attempt. Unnamed programs never leave
/// `Unnamed`, so their transaction carries no naming plan at all.
#[derive(Default)]
pub enum OperationNameEffects {
    #[default]
    Unnamed,
    Bound(BoundOperationNaming),
    Lowered(Box<CommitNamingPlan>),
}

impl WorthQueryRegisteredProviderEffects {
    /// Binds every declared output to the created reference its batch
    /// lowered, so Relational resolves each birth to its allocated record.
    ///
    /// Complexity: O(I log I + E log I) for I batch intents, E named outputs.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn bind_persistent_names(
        &mut self,
        naming: AuthoredOperationNaming,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if naming.is_empty() {
            return Ok(());
        }
        let mut created = BTreeMap::new();
        for intent in &self.batch.intents {
            let MutationIntent::Create(create) = intent else {
                continue;
            };
            let (subject, kind, key, target) = match create {
                CreateIntent::Entity(spec) => (
                    ApplicationNameSubjectKind::Entity,
                    spec.kind_id,
                    &spec.client_key,
                    PendingBirthTarget::CreatedEntity(CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: spec.client_key.clone(),
                    }),
                ),
                CreateIntent::Relation(spec) => (
                    ApplicationNameSubjectKind::Relation,
                    spec.kind_id,
                    &spec.client_key,
                    PendingBirthTarget::CreatedRelation(CreatedRelationRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: spec.client_key.clone(),
                        source: spec.source.clone(),
                        target: spec.target.clone(),
                    }),
                ),
                _ => continue,
            };
            if let Some(key) = key.as_raw_str() {
                created.insert((subject, kind, key.to_owned()), target);
            }
        }
        let bound = naming
            .try_bind(|output: OperationOutputKey| {
                created
                    .get(&(output.subject, output.kind, output.key.clone()))
                    .cloned()
                    .ok_or(WorthQueryOutputNameDenial::UnboundOutput {
                        subject: output.subject,
                        key: output.key,
                    })
            })
            .map_err(WorthQueryApplicationAttemptDenial::persistent_name_output)?;
        self.persistent_names = OperationNameEffects::Bound(bound);
        Ok(())
    }

    /// Lowers the bound declarations once, at the attempt's commit basis.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn lower_persistent_names(
        &mut self,
        lower: impl FnOnce(
            &BoundOperationNaming,
        ) -> Result<CommitNamingPlan, WorthQueryOutputNameDenial>,
    ) -> Result<(), WorthQueryOutputNameDenial> {
        if let OperationNameEffects::Bound(bound) = &self.persistent_names {
            self.persistent_names = OperationNameEffects::Lowered(Box::new(lower(bound)?));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn has_unlowered_names(
        &self,
    ) -> bool {
        matches!(self.persistent_names, OperationNameEffects::Bound(_))
    }

    /// The naming plan the attempt's transaction attaches, `None` when the
    /// program declared no output names.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn naming_plan(
        &self,
    ) -> Option<&CommitNamingPlan> {
        match &self.persistent_names {
            OperationNameEffects::Lowered(plan) => Some(plan),
            OperationNameEffects::Unnamed | OperationNameEffects::Bound(_) => None,
        }
    }
}
