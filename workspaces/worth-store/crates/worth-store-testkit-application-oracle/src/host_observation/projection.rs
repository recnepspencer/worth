//! Detached observations retain no owner, pin, performed receipt or service port.
use super::OracleRuntime;
use serde::{Deserialize, Serialize};
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_query_testkit_host::facade::host::application_home::schema::*;
use worth_relational::facade::{
    identity::{EntityId, LineageId, RelationId, VersionId},
    storage::RecordLifecycleState,
};
use worth_runtime_world::facade::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HeadProjection {
    pub owner: WorldOwnerCoordinate,
    pub branch: String,
    pub incarnation: u64,
    pub generation: u64,
    pub commit: u64,
}
pub fn owner(identity: RuntimeWorldOwnerIdentity) -> WorldOwnerCoordinate {
    match identity.durable() {
        Some(owner) => WorldOwnerCoordinate::Home { owner },
        None => WorldOwnerCoordinate::MemoryOnly {
            observation: identity.get(),
        },
    }
}
impl HeadProjection {
    pub fn from_live(head: &ProductBranchObservation) -> Self {
        Self {
            owner: owner(head.owner_identity()),
            branch: head.branch_identity().name().as_str().into(),
            incarnation: head.lifecycle_incarnation().ordinal(),
            generation: head.reference_generation().get(),
            commit: head.selected_commit().ordinal(),
        }
    }
    pub fn observe(head: &worth_foundational::facade::RecordedProductBranchReference) -> Self {
        use worth_foundational::facade::RecordedProductBranchReference as R;
        match head {
            R::Home(h) => Self {
                owner: WorldOwnerCoordinate::Home {
                    owner: h.branch().owner(),
                },
                branch: h.branch().name().into(),
                incarnation: h.incarnation().ordinal(),
                generation: h.generation().get(),
                commit: h.selected_commit().ordinal(),
            },
            R::Memory(h) => Self {
                owner: WorldOwnerCoordinate::MemoryOnly {
                    observation: h.owner(),
                },
                branch: h.branch().into(),
                incarnation: h.incarnation(),
                generation: h.generation(),
                commit: h.selected_commit(),
            },
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OccurrenceProjection {
    pub owner: WorldOwnerCoordinate,
    pub commit: u64,
    pub parent: Option<u64>,
    pub provenance: OccurrenceProvenance,
    pub relational_publication: Option<worth_relational::facade::history::RelationalCommitIdentity>,
    pub relational_fork: Option<worth_relational::facade::history::BranchId>,
    pub relational_changed: bool,
    pub signal_changed: bool,
    pub signal_movement: Option<super::occurrence::SignalMovement>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OccurrenceProvenance {
    Bootstrap(u64),
    Publication(u64),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StateProjection {
    pub idempotency: Vec<super::idempotency::IdempotencyProjection>,
    pub history_complete: bool,
    pub history_crossed_retired: bool,
    pub entities: Vec<GraphEntityProjection>,
    pub relations: Vec<GraphRelationProjection>,
    pub head: HeadProjection,
    pub occurrences: Vec<OccurrenceProjection>,
    pub custody: Vec<CustodyProjection>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEntityProjection {
    pub entity_id: EntityId,
    pub lineage_id: Option<LineageId>,
    pub kind_name: String,
    pub lifecycle: RecordLifecycleState,
    pub created_at_version: VersionId,
    pub retired_at_version: Option<VersionId>,
    pub authoritative_aspect_state: serde_json::Value,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphRelationProjection {
    pub relation_id: RelationId,
    pub source: EntityId,
    pub target: EntityId,
    pub kind_name: String,
    pub lifecycle: RecordLifecycleState,
    pub created_at_version: VersionId,
    pub retired_at_version: Option<VersionId>,
    pub authoritative_aspect_state: serde_json::Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustodyProjection {
    pub owner: WorldOwnerCoordinate,
    pub identity: u64,
    pub effects: Option<usize>,
    pub settlement: Option<bool>,
    pub cause: Option<super::custody::CustodyCause>,
    pub expected: Option<HeadProjection>,
    pub actions: Option<Vec<super::custody::CustodyAction>>,
}
pub fn observe(
    runtime: &OracleRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> StateProjection {
    let (entities, relations, symbols) = runtime
        .runtime()
        .application_oracle_graph_for_test(execution)
        .unwrap();
    let idempotency = super::idempotency::observe(&entities, &symbols);
    // The fixture's complete authored graph; platform idempotency/activation
    // rows are separately checked through actual outcome/receipt observations.
    let roles = [
        ExternalMapping::reference().name(),
        Principal::reference().name(),
        Account::reference().name(),
    ];
    let relation_roles = [
        MappingTarget::reference().name(),
        UnusedAccountPeer::reference().name(),
    ];
    let mut entities: Vec<_> = entities
        .into_iter()
        .filter(|r| roles.contains(&r.kind.kind_name.as_str()))
        .map(|r| GraphEntityProjection {
            entity_id: r.entity_id,
            lineage_id: r.lineage_id,
            kind_name: r.kind.kind_name,
            lifecycle: r.lifecycle,
            created_at_version: r.created_at_version,
            retired_at_version: r.retired_at_version,
            authoritative_aspect_state: super::semantic_text::state(
                &r.authoritative_aspect_state,
                &symbols,
            ),
        })
        .collect();
    entities.sort_by(|a, b| a.kind_name.cmp(&b.kind_name));
    let mut relations: Vec<_> = relations
        .into_iter()
        .filter(|r| relation_roles.contains(&r.kind.kind_name.as_str()))
        .map(|r| GraphRelationProjection {
            relation_id: r.relation_id,
            source: r.source,
            target: r.target,
            kind_name: r.kind.kind_name,
            lifecycle: r.lifecycle,
            created_at_version: r.created_at_version,
            retired_at_version: r.retired_at_version,
            authoritative_aspect_state: super::semantic_text::state(
                &r.authoritative_aspect_state,
                &symbols,
            ),
        })
        .collect();
    relations.sort_by(|a, b| a.kind_name.cmp(&b.kind_name));
    let head = runtime.runtime().application_oracle_head_for_test();
    let history = runtime.runtime().application_oracle_ancestry_for_test();

    let occurrences = history
        .occurrences
        .iter()
        .map(|entry| OccurrenceProjection {
            owner: owner(entry.identity.owner_identity()),
            commit: entry.identity.ordinal(),
            parent: match &entry.parent {
                CompositeCommitParent::Root => None,
                CompositeCommitParent::Ordinary(parent) => Some(parent.commit().ordinal()),
            },
            provenance: match &entry.provenance {
                CompositeCommitProvenance::Bootstrap(id) => {
                    OccurrenceProvenance::Bootstrap(id.ordinal())
                }
                CompositeCommitProvenance::Publication(id) => {
                    OccurrenceProvenance::Publication(id.ordinal())
                }
            },
            relational_publication: entry.relational_publication.clone(),
            relational_fork: entry.relational_fork.clone(),
            relational_changed: super::occurrence::changed(entry.relational_change),
            signal_changed: super::occurrence::changed(entry.signal_change),
            signal_movement: entry
                .signal_movement
                .map(super::occurrence::SignalMovement::observe),
        })
        .collect();
    let mut custody = Vec::new();
    let mut cursor = None;
    loop {
        let page = runtime
            .runtime()
            .product_publication_recovery_page(
                cursor.as_ref(),
                std::num::NonZeroUsize::new(64).unwrap(),
            )
            .unwrap();
        for row in page.rows() {
            let id = row.handle().identity();
            let (effects, settlement, cause, expected, actions) =
                if row.state() == RuntimeWorldRecoveryRecordState::Busy {
                    (None, None, None, None, None)
                } else {
                    let recovery = runtime
                        .runtime()
                        .readmit_product_publication_recovery(row.handle())
                        .unwrap();
                    let effect = recovery.inspect().unwrap();
                    (
                        Some(effect.owner_effect_count()),
                        Some(effect.relational_requires_settlement()),
                        Some(super::custody::CustodyCause::observe(effect.cause())),
                        Some(HeadProjection::from_live(effect.expected_product())),
                        Some(
                            effect
                                .next_actions()
                                .iter()
                                .copied()
                                .map(super::custody::CustodyAction::observe)
                                .collect(),
                        ),
                    )
                };
            custody.push(CustodyProjection {
                owner: owner(id.owner_identity()),
                identity: id.ordinal(),
                effects,
                settlement,
                cause,
                expected,
                actions,
            });
        }
        cursor = page.next_after().cloned();
        if cursor.is_none() {
            break;
        }
    }
    StateProjection {
        idempotency,
        history_complete: history.complete,
        history_crossed_retired: history.crossed_retired,
        entities,
        relations,
        head: HeadProjection::observe(&head),
        occurrences,
        custody,
    }
}
