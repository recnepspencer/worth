use worth_query::facade::runtime::WorthQueryEvidenceIdentityKey;
use worth_query::facade::{foundation::WorthQueryEntityIdentity, runtime::WorthQueryWorkspace};
use worth_ui_query_binding::{
    certification::{
        insert_projection_status, seeded_collection_projection_workspace,
        update_projection_identity, update_projection_status_batch,
        WorthUiCollectionProjectionSeedPosture,
    },
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionFactReceipt, UiCollectionProjectionOpenOutcome,
    UiCollectionProjectionRefreshOutcome, UiCollectionProjectionRefreshReceipt,
    UiCollectionProjectionRegistration, UiLiveCollectionProjection,
    UiLiveCollectionProjectionCloseOutcome, UiProjectionFieldRequirement, WorthUiQueryWorkspaceExt,
};

use super::oracle::ExpectedKeyedRows;

pub(super) enum WorldPosture {
    Complete,
    Partial,
    ResetOnly,
}

pub(super) struct CollectionProjectionWorld {
    workspace: WorthQueryWorkspace,
    live: Option<UiLiveCollectionProjection>,
    entities: Vec<WorthQueryEntityIdentity>,
    identities: Vec<WorthQueryEvidenceIdentityKey>,
    expected: ExpectedKeyedRows,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CollectionChangeCostEvidence {
    cardinality: usize,
    changed_rows: usize,
    ui: worth_ui_query_binding::UiCollectionProjectionWorkCounters,
    query: worth_ui_query_binding::WorthUiCollectionQueryWorkInspection,
}

impl CollectionChangeCostEvidence {
    pub(crate) const fn cardinality(self) -> usize {
        self.cardinality
    }

    pub(crate) const fn changed_rows(self) -> usize {
        self.changed_rows
    }

    pub(crate) const fn ui(self) -> worth_ui_query_binding::UiCollectionProjectionWorkCounters {
        self.ui
    }

    pub(crate) const fn query(
        self,
    ) -> worth_ui_query_binding::WorthUiCollectionQueryWorkInspection {
        self.query
    }
}

pub(crate) fn measure_changed_row_work(
    cardinality: usize,
    changed_rows: usize,
) -> CollectionChangeCostEvidence {
    assert!(changed_rows <= cardinality);
    let selected_rows = cardinality.min(512);
    let (mut world, initial) = CollectionProjectionWorld::open(
        cardinality,
        selected_rows as u32,
        WorldPosture::Complete,
        false,
    );
    assert_eq!(world.cardinality(), cardinality);
    assert_eq!(initial.work().rows_visited(), selected_rows);
    let changed = world.update_first(changed_rows);
    let receipt = world.refresh_receipt();
    world
        .expected()
        .assert_fact_rows(receipt.fact(), &world.expected().selected(&changed));
    let evidence = CollectionChangeCostEvidence {
        cardinality,
        changed_rows,
        ui: receipt.fact().work(),
        query: *receipt.query_work(),
    };
    world.close();
    evidence
}

impl CollectionProjectionWorld {
    pub(super) fn open(
        row_count: usize,
        max_rows: u32,
        posture: WorldPosture,
        requires_complete: bool,
    ) -> (Self, UiCollectionProjectionFactReceipt) {
        let seed_posture = match posture {
            WorldPosture::Complete => WorthUiCollectionProjectionSeedPosture::Complete,
            WorldPosture::Partial => WorthUiCollectionProjectionSeedPosture::Partial,
            WorldPosture::ResetOnly => WorthUiCollectionProjectionSeedPosture::ResetOnly,
        };
        let authored = authored_rows(row_count);
        let (mut workspace, entities) =
            seeded_collection_projection_workspace(authored.clone(), seed_posture);
        let mut expected = ExpectedKeyedRows::default();
        let identities = entities
            .iter()
            .zip(authored.iter())
            .map(|(entity, (_, value))| {
                let identity = correlation_identity(entity);
                expected.insert(identity, value.clone());
                identity
            })
            .collect();
        let (live, fact) = open_live(&mut workspace, max_rows, requires_complete);
        (
            Self {
                workspace,
                live: Some(live),
                entities,
                identities,
                expected,
            },
            fact,
        )
    }

    pub(super) fn expected(&self) -> &ExpectedKeyedRows {
        &self.expected
    }

    pub(super) fn identities(&self) -> &[WorthQueryEvidenceIdentityKey] {
        &self.identities
    }

    pub(super) fn cardinality(&self) -> usize {
        self.entities.len()
    }

    pub(super) fn update_first(&mut self, count: usize) -> Vec<WorthQueryEvidenceIdentityKey> {
        let selected = self.identities[..count].to_vec();
        let updates = self.entities[..count]
            .iter()
            .zip(selected.iter())
            .enumerate()
            .map(|(index, (entity, identity))| {
                let value = format!("Updated {index:05}");
                self.expected.update(identity, value.clone());
                (entity.clone(), value)
            })
            .collect();
        update_projection_status_batch(&mut self.workspace, updates);
        selected
    }

    pub(super) fn insert(
        &mut self,
        authored_identity: &str,
        value: &str,
    ) -> WorthQueryEvidenceIdentityKey {
        let entity = insert_projection_status(&mut self.workspace, authored_identity, value);
        let identity = correlation_identity(&entity);
        self.entities.push(entity);
        self.identities.push(identity);
        self.expected.insert(identity, value);
        identity
    }

    pub(super) fn remove(&mut self, index: usize) -> WorthQueryEvidenceIdentityKey {
        let entity = self.entities.remove(index);
        let identity = self.identities.remove(index);
        self.workspace
            .with_execution_request_mut(|execution, mut workspace| {
                workspace
                    .delete(execution, entity)
                    .expect("QP04 collection deletion")
            })
            .expect("workspace host call opens");
        self.expected.remove(&identity);
        identity
    }

    pub(super) fn reorder(
        &mut self,
        index: usize,
        authored_identity: &str,
    ) -> WorthQueryEvidenceIdentityKey {
        let entity = self.entities[index].clone();
        let identity = self.identities[index];
        update_projection_identity(&mut self.workspace, entity, authored_identity);
        identity
    }

    pub(super) fn refresh(&mut self) -> UiCollectionProjectionFactReceipt {
        self.refresh_receipt().into_fact()
    }

    pub(super) fn refresh_receipt(&mut self) -> UiCollectionProjectionRefreshReceipt {
        match self
            .workspace
            .with_execution_request_mut(|execution, mut workspace| {
                self.live
                    .as_mut()
                    .expect("live collection owner")
                    .refresh(execution, &mut workspace)
                    .expect("QP04 real Query refresh")
            })
            .expect("workspace host call opens")
        {
            UiCollectionProjectionRefreshOutcome::Applied(receipt) => receipt,
            UiCollectionProjectionRefreshOutcome::NoSemanticDelivery => {
                panic!("QP04 mutation must deliver semantic collection meaning")
            }
        }
    }

    pub(super) fn close(mut self) {
        let live = self.live.take().expect("live collection owner");
        let UiLiveCollectionProjectionCloseOutcome::Closed(closed) =
            live.close(&mut self.workspace)
        else {
            panic!("QP04 live collection owner must close");
        };
        assert!(closed.owner_terminal(), "the exact Query owner must retire");
    }
}

fn authored_rows(row_count: usize) -> Vec<(String, String)> {
    (0..row_count)
        .map(|index| (format!("pulse.{index:05}"), format!("Value {index:05}")))
        .collect()
}

fn correlation_identity(entity: &WorthQueryEntityIdentity) -> WorthQueryEvidenceIdentityKey {
    entity.evidence_identity().operational_key()
}

fn open_live(
    workspace: &mut WorthQueryWorkspace,
    max_rows: u32,
    requires_complete: bool,
) -> (
    UiLiveCollectionProjection,
    UiCollectionProjectionFactReceipt,
) {
    let installed = workspace.worth_ui().expect("WORTH UI domain installed");
    let registration = UiCollectionProjectionRegistration::text(
        installed
            .projection_view("certification.collection.qp04")
            .expect("QP04 installed collection view"),
        UiProjectionFieldRequirement::declared("identity.id").expect("row identity field"),
        [UiProjectionFieldRequirement::declared("status").expect("selected field")],
        requires_complete,
        true,
    )
    .expect("QP04 collection registration");
    let UiCollectionProjectionBindingAdmission::Ready(binding) = registration.admit(workspace)
    else {
        panic!("QP04 binding must admit");
    };
    let budget = UiCollectionProjectionBudget::new(max_rows, 131_072, 1, 8_388_608)
        .expect("QP04 collection budget");
    let UiCollectionProjectionOpenOutcome::Opened(opened) = workspace
        .with_execution_request_mut(|execution, mut workspace| {
            binding.open(execution, budget, workspace)
        })
        .expect("workspace host call opens")
    else {
        panic!("QP04 collection projection must open");
    };
    opened.into_parts()
}
