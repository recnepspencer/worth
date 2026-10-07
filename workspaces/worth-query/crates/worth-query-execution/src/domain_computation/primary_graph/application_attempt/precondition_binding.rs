mod canonical_identity;

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{AspectFieldLocator, AspectValue, CanonicalDigestId};
use worth_query_declaration::facade::application_schema::{
    ApplicationMutationPreconditionFamily, TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkEvidence, WorthQueryCompiledApplicationOperationContracts,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::InvalidationEditAdmission;
use canonical_identity::prepare_precondition_identity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryBoundMutationPrecondition {
    family: ApplicationMutationPreconditionFamily,
    entity: String,
    entity_id: EntityId,
    locator: AspectFieldLocator,
    expected_value: AspectValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryBoundMutationPreconditions {
    entries: Vec<WorthQueryBoundMutationPrecondition>,
    canonical_digest: Option<CanonicalDigestId>,
    canonical_work: WorthQueryCanonicalWorkEvidence,
    expected_version_count: usize,
    expected_fact_count: usize,
}

pub(in crate::domain_computation) fn bind_mutation_preconditions<Schema, Operation, Scope>(
    requested: TypedMutationPreconditions<Schema, Operation, Scope>,
    contracts: &WorthQueryCompiledApplicationOperationContracts,
    scope_entity_name: &str,
    scope_entity_id: EntityId,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<WorthQueryBoundMutationPreconditions, ()> {
    let mut entries = requested.into_entries();
    if entries.is_empty() {
        return Ok(empty_bound_preconditions());
    }
    entries.sort_by(|left, right| left.target().cmp(right.target()));
    if entries
        .windows(2)
        .any(|pair| pair[0].target() == pair[1].target())
    {
        return Err(());
    }
    let installed = contracts
        .mutation_preconditions()
        .iter()
        .map(|precondition| precondition.target())
        .collect::<BTreeSet<_>>();
    let mut expected_version_count = 0usize;
    let mut expected_fact_count = 0usize;
    let mut bound = Vec::with_capacity(entries.len());
    for entry in &entries {
        let target = entry.target();
        if target.entity() != scope_entity_name || !installed.contains(target) {
            return Err(());
        }
        let locator = layout
            .field_locator(target.entity(), target.aspect(), target.field_name())
            .cloned()
            .ok_or(())?;
        match target.family() {
            ApplicationMutationPreconditionFamily::ExpectedVersion => {
                expected_version_count = expected_version_count.checked_add(1).ok_or(())?;
            }
            ApplicationMutationPreconditionFamily::ExpectedFact => {
                expected_fact_count = expected_fact_count.checked_add(1).ok_or(())?;
            }
        }
        bound.push(WorthQueryBoundMutationPrecondition {
            family: target.family(),
            entity: target.entity().to_owned(),
            entity_id: scope_entity_id,
            locator,
            expected_value: entry.expected_value().clone(),
        });
    }
    let (canonical_digest, canonical_work) = if entries.is_empty() {
        (None, WorthQueryCanonicalWorkEvidence::zero())
    } else {
        let budget = contracts.precondition_canonical_work_budget().ok_or(())?;
        let canonical = prepare_precondition_identity(&entries, scope_entity_id, budget)?;
        (Some(canonical.digest), canonical.work)
    };
    Ok(WorthQueryBoundMutationPreconditions {
        entries: bound,
        canonical_digest,
        canonical_work,
        expected_version_count,
        expected_fact_count,
    })
}

/// The producer's default preconditions have no selected rows. This is the
/// same empty acceptance branch as the ordinary binder, issued before its
/// unrelated installed-target collection. A nonempty selected request has no
/// admitted binding route and refuses before operation effects.
pub(in crate::domain_computation) fn bind_empty_mutation_preconditions_admitted<
    Schema,
    Operation,
    Scope,
>(
    requested: TypedMutationPreconditions<Schema, Operation, Scope>,
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryBoundMutationPreconditions, EmptyMutationPreconditionAdmissionStop> {
    let work = std::mem::size_of::<WorthQueryBoundMutationPreconditions>()
        .checked_add(3)
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(EmptyMutationPreconditionAdmissionStop::AccountingOverflow)?;
    admission
        .charge_external_work(work)
        .map_err(EmptyMutationPreconditionAdmissionStop::Admission)?;
    if !requested.entries().is_empty() {
        return Err(EmptyMutationPreconditionAdmissionStop::Nonempty);
    }
    Ok(empty_bound_preconditions())
}

#[derive(Debug)]
pub(in crate::domain_computation) enum EmptyMutationPreconditionAdmissionStop {
    Admission(CompanionPreflightStop),
    AccountingOverflow,
    Nonempty,
}

fn empty_bound_preconditions() -> WorthQueryBoundMutationPreconditions {
    WorthQueryBoundMutationPreconditions {
        entries: Vec::new(),
        canonical_digest: None,
        canonical_work: WorthQueryCanonicalWorkEvidence::zero(),
        expected_version_count: 0,
        expected_fact_count: 0,
    }
}

impl WorthQueryBoundMutationPreconditions {
    pub(in crate::domain_computation::primary_graph) fn validate_observations(
        &self,
        facts: &BTreeMap<WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact>,
    ) -> Result<(), ()> {
        self.entries.iter().try_for_each(|precondition| {
            let key = WorthQueryApplicationFactKey::Field {
                entity: precondition.entity.clone(),
                entity_id: precondition.entity_id,
                locator: precondition.locator.clone(),
            };
            facts
                .get(&key)
                .filter(|fact| {
                    fact.observed_field_value()
                        .is_some_and(|value| value == &precondition.expected_value)
                })
                .map(|_| ())
                .ok_or(())
        })
    }

    pub(in crate::domain_computation) fn identity(&self) -> Option<&[u8; 32]> {
        self.canonical_digest.as_ref().map(CanonicalDigestId::bytes)
    }

    pub(in crate::domain_computation) const fn canonical_work(
        &self,
    ) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }

    pub(in crate::domain_computation) const fn expected_version_count(&self) -> usize {
        self.expected_version_count
    }

    pub(in crate::domain_computation) const fn expected_fact_count(&self) -> usize {
        self.expected_fact_count
    }
}
