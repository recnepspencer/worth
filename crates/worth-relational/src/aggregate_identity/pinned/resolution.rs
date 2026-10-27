//! Current identity follows validated merge coordinates, never split edges or rescans.
use super::super::{cohorts::StoredAggregate, funding::AggregateWork, AggregateLineageEvent};
use super::*;
use crate::schema::data::AggregateIdentityContractDeclaration;
use std::collections::BTreeSet;
use worth_foundational::facade::AggregateIdentity;

pub(super) struct Resolution {
    pub(super) incident: BTreeSet<AggregateLineageEvent>,
    pub(super) steps: Vec<AggregateLineageEvent>,
    pub(super) latest: Option<AggregateIdentity>,
}

pub(super) fn resolve(
    view: &InvariantStateView<'_>,
    mut current: StoredAggregate,
    contract: &AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Resolution> {
    let mut ids = BTreeSet::new();
    let mut rows = BTreeSet::new();
    let mut incident = BTreeSet::new();
    let mut steps = Vec::new();
    let mut initial = true;
    loop {
        work.checkpoint(1)?;
        if !ids.insert(current.record.identity) || !rows.insert(current.entity) {
            return work.invalid(PlanDenial::LineageCycle);
        }
        let next =
            super::super::stored_lineage::visit(view, &current, contract, work, |event, _| {
                if initial {
                    incident.insert(event);
                }
                Ok(())
            })?;
        match next {
            Some(next) => {
                steps.push(next.event);
                current = next.target;
                initial = false;
            }
            None => {
                return Ok(Resolution {
                    incident,
                    steps,
                    latest: current
                        .record
                        .current_anchor
                        .map(|_| current.record.identity),
                })
            }
        }
    }
}
