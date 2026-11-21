//! Prepared field encoding for fragment record writes.

use super::{
    WorthQueryApplicationCreationPartition as Partition,
    WorthQueryApplicationEffectEntity as EffectEntity,
    WorthQueryApplicationEffectProgramBuilder as Builder,
};
use crate::domain_computation::primary_graph::fragment_view::{
    ErasedCandidate, FragmentDenial, WritePort,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::fragment::RelationDefinition;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::{
    AspectDefinition, RecordFieldDefinition,
};

use super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};

pub fn prepare_fields(
    definition: &AspectDefinition,
    mut fields: BTreeMap<String, AspectValue>,
) -> Result<Vec<(&RecordFieldDefinition, Option<AspectValue>)>, Denial> {
    let mut prepared = Vec::new();
    prepared
        .try_reserve(definition.fields().len())
        .map_err(|_| Denial::new(Kind::AllocationDenied, definition.path()))?;
    for field in definition.fields() {
        let locus = field.recipe().locus();
        let path = format!("{}.{}", definition.path(), locus.field());
        let value = fields.remove(&path);
        if value
            .as_ref()
            .is_some_and(|value| value.value_family() != field.recipe().scalar_family())
        {
            return Err(Denial::new(Kind::InvalidEffectValue, &path));
        }
        if value.is_none()
            && field.presence()
                == worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Required
        {
            return Err(Denial::new(Kind::InvalidEffectValue, &path));
        }
        prepared.push((field, value));
    }
    if let Some((path, _)) = fields.first_key_value() {
        return Err(Denial::new(Kind::InvalidEffectValue, path));
    }
    Ok(prepared)
}

impl<S, O, I, Scope> WritePort for Builder<S, O, I, Scope> {
    fn case_record(
        &mut self,
        target: &ErasedCandidate,
        aspect: &AspectDefinition,
        group: &worth_query_declaration::facade::application_schema::fragment::GroupDefinition,
        cases: &[&AspectDefinition],
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial> {
        self.write_case_record(target, aspect, group, cases, fields)
    }
    fn target(
        &self,
        target: &crate::domain_computation::primary_graph::WorthQueryInvariantMutationTarget<
            (),
            (),
        >,
    ) -> Result<ErasedCandidate, FragmentDenial> {
        let native = target.fragment_schema::<S>();
        Ok(erase(&self.projected_entity(&native)?))
    }

    fn create(&mut self, entity: &str, key: &str) -> Result<ErasedCandidate, FragmentDenial> {
        let native =
            self.create_entity_named_at::<()>(entity, key.to_owned(), Partition::Issued)?;
        Ok(erase(&native))
    }

    fn record(
        &mut self,
        target: &ErasedCandidate,
        definition: &AspectDefinition,
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial> {
        self.native_effects().record(target, definition, fields)
    }

    fn link(
        &mut self,
        from: &ErasedCandidate,
        relation: &RelationDefinition,
        to: &ErasedCandidate,
    ) -> Result<(), FragmentDenial> {
        self.native_effects().link_fragment(from, relation, to)
    }
}

fn erase<S, E>(target: &EffectEntity<S, E>) -> ErasedCandidate {
    use worth_query_execution_lineage::facade::ports::EffectEntityInternalPort as _;
    ErasedCandidate {
        reference: target.reference().clone(),
        entity: target.entity().to_owned(),
        created_effect: target.created_effect(),
        program: Arc::clone(target.program()),
    }
}
