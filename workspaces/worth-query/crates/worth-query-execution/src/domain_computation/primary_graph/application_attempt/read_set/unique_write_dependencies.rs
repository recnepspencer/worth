//! Intrinsic indexed dependencies earned by admitted unique writes. Observe
//! normalized values at the retained basis before signing/sealing, not through
//! supplier decision permission. Lowering still decides collisions.

use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
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
const UNIQUE_VALUE_CANDIDATE_LIMIT: usize = 2;

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
    /// Appends one indexed selection for each distinct written unique value.
    /// Replacement fact backing is admitted before copying retained facts.
    pub(in crate::domain_computation::primary_graph) fn observe_unique_write_values(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: &[WorthQueryApplicationRealizedEffect],
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if self.lease.layout.unique_fields().is_empty() {
            return Ok(());
        }
        let appended = observe_unique_values(
            &self.facts,
            self.lease.layout.unique_fields(),
            effects,
            |entity| self.lease.layout.entity_kind(entity),
            |index, kind, locator, value| {
                self.lease.handle().with_runtime(execution, |runtime| {
                    observe_indexed_entity_selection(
                        runtime,
                        execution,
                        self.lease.snapshot(),
                        index,
                        kind,
                        locator,
                        value,
                        UNIQUE_VALUE_CANDIDATE_LIMIT,
                    )
                })
            },
        )?;
        self.append_completed_facts(appended, allocation_policy)?;
        Ok(())
    }
}

fn observe_unique_values(
    facts: &(impl FactSequence + ?Sized),
    unique: crate::domain_computation::primary_graph::schema_layout::WorthQueryUniqueFields<'_>,
    effects: &[WorthQueryApplicationRealizedEffect],
    kind_of: impl Fn(&str) -> Option<KindId>,
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
    for (kind, locator, value) in writes::values(effects, kind_of)? {
        let index_id = match unique.index(kind, locator) {
            None => continue,
            Some(WorthQueryUniqueFieldIndex::Unavailable) => return Err(unavailable(locator)),
            Some(WorthQueryUniqueFieldIndex::Installed(index_id)) => index_id,
        };
        let key = (
            index_id,
            kind,
            locator.clone(),
            prepare_aspect_value_identity_basis(value),
        );
        if !observed.insert(key) {
            continue;
        }
        let selection = observe(index_id, kind, locator.clone(), value.clone());
        appended.push(selection.map_err(|refusal| match refusal {
            // More holders than the limit: the value is held.
            WorthQueryIndexedSelectionRefusal::Overflowed => denial(
                WorthQueryApplicationAttemptDenialKind::UniqueValueTaken,
                format!("{locator:?}"),
            ),
            WorthQueryIndexedSelectionRefusal::Execution(cause) => {
                WorthQueryApplicationAttemptDenial::relational_read_stopped(
                    cause,
                    "unique write value",
                )
            }
            WorthQueryIndexedSelectionRefusal::Unavailable => unavailable(locator),
        })?);
    }
    Ok(appended)
}
#[cfg(test)]
mod tests;
mod writes;

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
