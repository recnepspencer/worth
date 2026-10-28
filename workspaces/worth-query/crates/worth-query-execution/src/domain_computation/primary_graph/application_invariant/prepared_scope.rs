use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::runtime::CustomInvariantReadBudget;

use super::{WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind};

mod retention_extent;

#[derive(Default)]
pub(super) struct PlanningAdmission {
    entities: BTreeSet<EntityId>,
    relations: BTreeSet<RelationId>,
}

pub(super) struct ApplicationInvariantAdmission {
    entities: BTreeSet<EntityId>,
    relations: BTreeSet<RelationId>,
}

#[derive(Clone)]
pub(super) enum ApplicationInvariantAdmissionAccess {
    Planning(Arc<Mutex<PlanningAdmission>>),
    Evaluation(Arc<ApplicationInvariantAdmission>),
}

impl ApplicationInvariantAdmissionAccess {
    pub(super) fn planning(
        budget: CustomInvariantReadBudget<'_>,
    ) -> Result<Self, WorthQueryInvariantAccessDenial> {
        budget
            .claim_retained(retention_extent::planning_arc_bytes())
            .map_err(retention_denial)?;
        Ok(Self::Planning(Arc::new(Mutex::new(
            PlanningAdmission::default(),
        ))))
    }

    pub(super) fn freeze(
        &self,
        budget: CustomInvariantReadBudget<'_>,
    ) -> Result<Arc<ApplicationInvariantAdmission>, WorthQueryInvariantAccessDenial> {
        let Self::Planning(admission) = self else {
            unreachable!("only planning admission can be frozen");
        };
        let admission = admission
            .lock()
            .expect("application invariant admission mutex must not be poisoned");
        budget
            .claim_retained(retention_extent::frozen_bytes(
                admission.entities.len(),
                admission.relations.len(),
            ))
            .map_err(retention_denial)?;
        Ok(Arc::new(ApplicationInvariantAdmission {
            entities: admission.entities.clone(),
            relations: admission.relations.clone(),
        }))
    }

    pub(super) fn entity(
        &self,
        entity_id: EntityId,
        budget: CustomInvariantReadBudget<'_>,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        budget.check_live().map_err(retention_denial)?;
        match self {
            Self::Planning(admission) => {
                let mut admission = admission
                    .lock()
                    .expect("application invariant admission mutex must not be poisoned");
                if !admission.entities.contains(&entity_id) {
                    budget
                        .claim_retained(retention_extent::entity_entry_bytes())
                        .map_err(retention_denial)?;
                    admission.entities.insert(entity_id);
                }
                Ok(())
            }
            Self::Evaluation(admission) if admission.entities.contains(&entity_id) => Ok(()),
            Self::Evaluation(_) => Err(denial("entity")),
        }
    }

    pub(super) fn relation(
        &self,
        relation_id: RelationId,
        budget: CustomInvariantReadBudget<'_>,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        budget.check_live().map_err(retention_denial)?;
        match self {
            Self::Planning(admission) => {
                let mut admission = admission
                    .lock()
                    .expect("application invariant admission mutex must not be poisoned");
                if !admission.relations.contains(&relation_id) {
                    budget
                        .claim_retained(retention_extent::relation_entry_bytes())
                        .map_err(retention_denial)?;
                    admission.relations.insert(relation_id);
                }
                Ok(())
            }
            Self::Evaluation(admission) if admission.relations.contains(&relation_id) => Ok(()),
            Self::Evaluation(_) => Err(denial("relation")),
        }
    }
}

fn retention_denial(
    cause: worth_relational::facade::runtime::StructuralWalkDenialKind,
) -> WorthQueryInvariantAccessDenial {
    WorthQueryInvariantAccessDenial::from_native_read(cause, "prepared read admission")
}

fn denial(subject: &str) -> WorthQueryInvariantAccessDenial {
    WorthQueryInvariantAccessDenial::new(
        WorthQueryInvariantAccessDenialKind::OutsidePreparedScope,
        subject,
    )
}
