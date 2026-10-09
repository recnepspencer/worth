//! The observed absence a delegation's child create needs. The activation
//! program writes the child's identity, a unique field, so the read set must
//! hold that value's indexed selection; the unique law then decides at
//! lowering. A selection the decision already observed is not repeated.

use std::collections::BTreeSet;

use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectFieldLocator, CanonicalAspectValueIdentityBasis,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;

use super::super::effect_program::WorthQueryApplicationRealizedEffect;
use super::super::effect_validation::denial;
use super::super::fact::{observe_indexed_entity_selection, WorthQueryIndexedSelectionRefusal};
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryUniqueFieldIndex;

/// Two candidates tell a free value from a taken one, and a held value from
/// a duplicated one.
const CHILD_IDENTITY_CANDIDATE_LIMIT: usize = 2;

type SelectionKey = (
    DerivedIndexId,
    KindId,
    AspectFieldLocator,
    CanonicalAspectValueIdentityBasis,
);

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    /// Appends one indexed selection for each distinct created unique value.
    /// Replacement fact backing is admitted before copying retained facts.
    pub(super) fn observe_created_unique_values(
        &mut self,
        effects: &[WorthQueryApplicationRealizedEffect],
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let appended = observe_created_values(
            &self.facts,
            self.lease.layout.unique_fields(),
            effects,
            |index, kind, locator, value| {
                self.lease.handle().with_runtime(|runtime| {
                    observe_indexed_entity_selection(
                        runtime,
                        self.lease.snapshot(),
                        index,
                        kind,
                        locator,
                        value,
                        CHILD_IDENTITY_CANDIDATE_LIMIT,
                    )
                })
            },
        )?;
        let operation = self.admission.operation();
        let count = self
            .facts
            .len()
            .checked_add(appended.len())
            .ok_or_else(|| {
                super::super::retained_decision_facts::StoreDenial::Representability
                    .into_attempt_denial(operation)
            })?;
        self.facts = super::super::read_set::admit_array(
            count,
            self.facts.iter().cloned().chain(appended),
            allocation_policy,
            operation,
            || {
                self.admission
                    .validate_current_authority()
                    .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
            },
        )?;
        Ok(())
    }
}

fn observe_created_values(
    facts: &[WorthQueryApplicationObservedFact],
    unique: crate::domain_computation::primary_graph::schema_layout::WorthQueryUniqueFields<'_>,
    effects: &[WorthQueryApplicationRealizedEffect],
    mut observe: impl FnMut(
        DerivedIndexId,
        KindId,
        AspectFieldLocator,
        worth_foundational::facade::AspectValue,
    ) -> Result<
        WorthQueryApplicationObservedFact,
        WorthQueryIndexedSelectionRefusal,
    >,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    let mut observed = facts
        .iter()
        .filter_map(selection_key)
        .collect::<BTreeSet<_>>();
    let mut appended = Vec::new();
    for effect in effects {
        let WorthQueryApplicationRealizedEffect::CreateEntity { kind, fields, .. } = effect else {
            continue;
        };
        for (locator, value) in fields {
            let index_id = match unique.index(*kind, locator) {
                None => continue,
                Some(WorthQueryUniqueFieldIndex::Unavailable) => return Err(unavailable(locator)),
                Some(WorthQueryUniqueFieldIndex::Installed(index_id)) => index_id,
            };
            let key = (
                index_id,
                *kind,
                locator.clone(),
                prepare_aspect_value_identity_basis(value),
            );
            if !observed.insert(key) {
                continue;
            }
            let selection = observe(index_id, *kind, locator.clone(), value.clone());
            appended.push(selection.map_err(|refusal| match refusal {
                // More holders than the limit: the value is held.
                WorthQueryIndexedSelectionRefusal::Overflowed => denial(
                    WorthQueryApplicationAttemptDenialKind::UniqueValueTaken,
                    format!("{locator:?}"),
                ),
                WorthQueryIndexedSelectionRefusal::Unavailable => unavailable(locator),
            })?);
        }
    }
    Ok(appended)
}
#[cfg(test)]
mod tests;

fn selection_key(fact: &WorthQueryApplicationObservedFact) -> Option<SelectionKey> {
    match fact {
        WorthQueryApplicationObservedFact::IndexedEntitySelection {
            index_id,
            entity_kind,
            locator,
            value,
            ..
        } => Some((
            *index_id,
            *entity_kind,
            locator.clone(),
            prepare_aspect_value_identity_basis(value),
        )),
        _ => None,
    }
}

fn unavailable(locator: &AspectFieldLocator) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::UniqueIndexUnavailable,
        format!("{locator:?}"),
    )
}
