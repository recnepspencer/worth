//! A row preflights every scalar, then uses the ordinary field effect machinery.

use super::WorthQueryApplicationAttemptDenial as Denial;
use super::WorthQueryApplicationAttemptDenialKind as Kind;
use super::WorthQueryApplicationEffectEntity as EffectEntity;
use super::WorthQueryApplicationEffectProgramBuilder as Builder;
use crate::domain_computation::primary_graph::fragment_view::ErasedCandidate;
use crate::domain_computation::primary_graph::fragment_view::FragmentDenial;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::AspectDefinition;
use worth_query_declaration::facade::application_schema::fragment::RecordFieldDefinition;

impl<S: worth_query_installation::facade::ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>
{
    pub fn fragment_candidate<
        'a,
        F: worth_query_declaration::facade::application_schema::fragment::Fragment,
        O,
        I,
        Scope,
    >(
        &'a self,
        candidate: &'a mut Builder<S, O, I, Scope>,
        marker: F,
    ) -> Result<
        crate::domain_computation::primary_graph::fragment_view::View<
            'a,
            F,
            crate::domain_computation::primary_graph::fragment_view::Writing<'a>,
        >,
        FragmentDenial,
    > {
        if !candidate.read_set.admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(Denial::new(
                Kind::ForeignApplication,
                candidate.read_set.admission.operation(),
            )
            .into());
        }
        let definition = self
            .installed_schema
            .value_bindings()
            .fragment(marker)
            .ok_or(FragmentDenial::NotInstalled { fragment: F::NAME })?;
        Ok(
            crate::domain_computation::primary_graph::fragment_view::View::writing(
                definition, candidate,
            ),
        )
    }
}


fn erase<S, E>(target: &EffectEntity<S, E>) -> ErasedCandidate {
    ErasedCandidate {
        reference: target.reference.clone(),
        entity: target.entity.clone(),
        created_effect: target.created_effect,
        program: Arc::clone(&target.program),
    }
}
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
        if value.is_none() && field.presence() == worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Required {
            return Err(Denial::new(Kind::InvalidEffectValue, &path));
        }
        prepared.push((field, value));
    }
    if let Some((path, _)) = fields.first_key_value() {
        return Err(Denial::new(Kind::InvalidEffectValue, path));
    }
    Ok(prepared)
}
