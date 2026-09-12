use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use worth_relational::facade::identity::{EntityId, RelationId};

use super::{WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind};

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
    pub(super) fn planning() -> Self {
        Self::Planning(Arc::new(Mutex::new(PlanningAdmission::default())))
    }

    pub(super) fn freeze(&self) -> Arc<ApplicationInvariantAdmission> {
        let Self::Planning(admission) = self else {
            unreachable!("only planning admission can be frozen");
        };
        let admission = admission
            .lock()
            .expect("application invariant admission mutex must not be poisoned");
        Arc::new(ApplicationInvariantAdmission {
            entities: admission.entities.clone(),
            relations: admission.relations.clone(),
        })
    }

    pub(super) fn entity(
        &self,
        entity_id: EntityId,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        match self {
            Self::Planning(admission) => {
                admission
                    .lock()
                    .expect("application invariant admission mutex must not be poisoned")
                    .entities
                    .insert(entity_id);
                Ok(())
            }
            Self::Evaluation(admission) if admission.entities.contains(&entity_id) => Ok(()),
            Self::Evaluation(_) => Err(denial("entity")),
        }
    }

    pub(super) fn relation(
        &self,
        relation_id: RelationId,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        match self {
            Self::Planning(admission) => {
                admission
                    .lock()
                    .expect("application invariant admission mutex must not be poisoned")
                    .relations
                    .insert(relation_id);
                Ok(())
            }
            Self::Evaluation(admission) if admission.relations.contains(&relation_id) => Ok(()),
            Self::Evaluation(_) => Err(denial("relation")),
        }
    }
}

fn denial(subject: &str) -> WorthQueryInvariantAccessDenial {
    WorthQueryInvariantAccessDenial::new(
        WorthQueryInvariantAccessDenialKind::OutsidePreparedScope,
        subject,
    )
}
