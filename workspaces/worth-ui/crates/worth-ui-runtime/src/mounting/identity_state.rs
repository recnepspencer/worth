use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::facade::WorthUiHostSessionIdentity;
use crate::graph::UiGraphNodeIdentity;
use worth_ui_host_contract::{
    UiHostSurfaceRegistrationRequest, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedProjectionAudience, UiSemanticSurfaceIdentity,
};

use super::{
    UiMountedGraphWorldIdentity, UiMountedIdentityBasis, UiMountedIdentityDenial,
    UiSurfaceBindingIdentityView,
};

static NEXT_WORLD: AtomicU64 = AtomicU64::new(1);
static NEXT_STATE_REVISION: AtomicU64 = AtomicU64::new(1);
const SEMANTIC_SURFACE_LIMIT: usize = 256;
const RETIRED_INSTANCE_LIMIT: usize = 256;

mod appearance_receipt_basis;
mod frame_lifecycle;
mod graph_replacement;
mod instance_lifecycle;
mod interaction_affinity;
mod layout_reconstruction;
mod presentation_attribution;
pub(crate) mod surface_lifecycle;

pub(crate) use interaction_affinity::{
    UiCurrentHitTarget, UiCurrentHitTargetAffinityDenial, UiCurrentInteractionAffinity,
    UiMountedIncarnationAffinityInput, UiMountedInteractionAffinityInput,
};

#[derive(Clone, Debug)]
struct MountedInstanceRecord {
    basis: UiMountedIdentityBasis,
}

#[derive(Clone, Copy, Debug)]
struct SurfaceBindingRecord {
    view: UiSurfaceBindingIdentityView,
    request: UiHostSurfaceRegistrationRequest,
}

#[derive(Clone)]
pub(crate) struct UiMountedIdentityFrameCandidate {
    receipt_basis: super::UiMountedNodeReceiptBasis,
}

pub(crate) struct UiAuthorityAdmittedMountedFrame {
    frame: super::UiPreparedMountedFrame,
}

pub(crate) struct UiMountedProjectionAffectedInstances {
    instances: Box<[UiMountedInstanceIdentity]>,
    index_entries_touched: usize,
}

pub(crate) struct UiMountedIdentityState {
    world_identity: UiMountedGraphWorldIdentity,
    host_session_identity: WorthUiHostSessionIdentity,
    semantic_surfaces: BTreeMap<UiSemanticSurfaceIdentity, UiMountedProjectionAudience>,
    bindings: BTreeMap<UiSemanticSurfaceIdentity, SurfaceBindingRecord>,
    instances: BTreeMap<UiMountedInstanceIdentity, MountedInstanceRecord>,
    retired_instances: BTreeSet<UiMountedInstanceIdentity>,
    retirement_order: VecDeque<UiMountedInstanceIdentity>,
    by_graph: BTreeMap<UiGraphNodeIdentity, BTreeSet<UiMountedInstanceIdentity>>,
    visible_order: crate::runtime::persistent_index::UiPersistentOrder<UiMountedInstanceIdentity>,
    mounted_instance_membership:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiMountedInstanceIdentity>,
    current_frame: Option<UiMountedFrameIdentity>,
    current_receipt_basis: Option<super::UiMountedNodeReceiptBasis>,
    current_projection: Option<std::rc::Rc<super::UiMountedProjectionFrame>>,
    current_manifest: Option<worth_ui_host_contract::UiMountedFrameManifest>,
    current_core: Option<worth_ui_host_contract::UiMountedFrameCanonicalCore>,
    current_publication: Option<super::UiMountedFramePublicationReceipt>,
    current_trace_source:
        Option<crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource>,
    current_reuse_contract: Option<super::UiMountedFrameReuseContract>,
    pending_projection_changes: super::UiMountedProjectionChanges,
    peak_qualified_layouts: usize,
    semantic_revision: u64,
    binding_revision: u64,
}

impl UiMountedIdentityState {
    pub(crate) fn world_identity(&self) -> UiMountedGraphWorldIdentity {
        self.world_identity
    }

    pub(crate) fn new(
        host_session_identity: WorthUiHostSessionIdentity,
    ) -> Result<Self, UiMountedIdentityDenial> {
        let semantic_revision = next(&NEXT_STATE_REVISION)?;
        let binding_revision = next(&NEXT_STATE_REVISION)?;
        Ok(Self {
            world_identity: UiMountedGraphWorldIdentity::new(next(&NEXT_WORLD)?),
            host_session_identity,
            semantic_surfaces: BTreeMap::new(),
            bindings: BTreeMap::new(),
            instances: BTreeMap::new(),
            retired_instances: BTreeSet::new(),
            retirement_order: VecDeque::new(),
            by_graph: BTreeMap::new(),
            visible_order: Default::default(),
            mounted_instance_membership: Default::default(),
            current_frame: None,
            current_receipt_basis: None,
            current_projection: None,
            current_manifest: None,
            current_core: None,
            current_publication: None,
            current_trace_source: None,
            current_reuse_contract: None,
            pending_projection_changes: Default::default(),
            peak_qualified_layouts: 0,
            semantic_revision,
            binding_revision,
        })
    }

    pub(crate) fn create_semantic_surface(
        &mut self,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        self.create_semantic_surface_for(UiMountedProjectionAudience::full())
    }

    pub(crate) fn create_semantic_surface_for(
        &mut self,
        audience: UiMountedProjectionAudience,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        if self.semantic_surfaces.len() >= SEMANTIC_SURFACE_LIMIT {
            return Err(UiMountedIdentityDenial::SemanticSurfaceCapacityExceeded);
        }
        let identity = UiSemanticSurfaceIdentity::mint_unbound()
            .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)?;
        let semantic_revision = next(&NEXT_STATE_REVISION)?;
        self.semantic_surfaces.insert(identity, audience);
        self.semantic_revision = semantic_revision;
        Ok(identity)
    }

    fn require_surface(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.semantic_surfaces
            .contains_key(&surface)
            .then_some(())
            .ok_or(UiMountedIdentityDenial::UnknownSemanticSurface)
    }

    fn remember_retirement(&mut self, identity: UiMountedInstanceIdentity) {
        if !self.retired_instances.insert(identity) {
            return;
        }
        self.retirement_order.push_back(identity);
        if self.retirement_order.len() > RETIRED_INSTANCE_LIMIT {
            let expired = self
                .retirement_order
                .pop_front()
                .expect("an over-limit retirement queue is non-empty");
            self.retired_instances.remove(&expired);
        }
    }

    pub(crate) fn seal_reuse_contract(
        &self,
        basis: super::UiMountedFrameReuseExternalBasis,
    ) -> super::UiMountedFrameReuseContract {
        super::UiMountedFrameReuseContract::seal(
            basis,
            self.world_identity.diagnostic_value(),
            self.semantic_revision,
            self.binding_revision,
        )
    }

    pub(crate) fn projection_change_snapshot(&self) -> super::UiMountedProjectionChangeSnapshot {
        self.pending_projection_changes
            .snapshot(self.semantic_revision, self.binding_revision)
    }

    fn commit_projection_changes(
        &mut self,
        snapshot: &super::UiMountedProjectionChangeSnapshot,
    ) -> bool {
        if !snapshot.matches(self.semantic_revision, self.binding_revision) {
            return false;
        }
        snapshot.commit_into(&mut self.pending_projection_changes);
        true
    }

    pub(crate) fn current_projection(&self) -> Option<&super::UiMountedProjectionFrame> {
        self.current_projection.as_deref()
    }

    pub(crate) fn focus_participation_snapshot(
        &self,
    ) -> Option<super::UiMountedFocusParticipationSnapshot> {
        let projection = self.current_projection.as_ref()?;
        let receipts = self.current_receipt_basis.as_ref()?;
        Some(super::UiMountedFocusParticipationSnapshot::from_projection(
            projection, receipts,
        ))
    }

    pub(crate) fn current_allocation_truth_revision(&self) -> Option<u64> {
        self.current_core
            .as_ref()
            .map(|core| core.allocation_truth_revision())
    }

    pub(crate) fn projection_instance(
        &self,
        identity: UiMountedInstanceIdentity,
    ) -> Option<super::UiMountedInstanceIdentityView> {
        self.instances
            .get(&identity)
            .map(|record| super::UiMountedInstanceIdentityView::new(identity, record.basis.clone()))
    }

    pub(crate) fn projection_instances(
        &self,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) -> Vec<super::UiMountedInstanceIdentityView> {
        self.visible_order
            .iter()
            .filter_map(|identity| self.projection_instance(*identity))
            .filter(|instance| surfaces.contains(&instance.basis().semantic_surface_identity()))
            .collect()
    }

    pub(crate) fn try_projection_instances_for_graph_nodes(
        &self,
        graph_nodes: &[UiGraphNodeIdentity],
    ) -> Option<UiMountedProjectionAffectedInstances> {
        let mut instances = BTreeSet::new();
        let mut index_entries_touched = 0usize;
        for graph_node in graph_nodes {
            index_entries_touched = index_entries_touched.checked_add(1)?;
            if let Some(mounted) = self.by_graph.get(graph_node) {
                index_entries_touched = index_entries_touched.checked_add(mounted.len())?;
                instances.extend(mounted.iter().copied());
            }
        }
        Some(UiMountedProjectionAffectedInstances {
            instances: instances.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            index_entries_touched,
        })
    }

    pub(crate) fn projection_order(
        &self,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) -> Vec<UiMountedInstanceIdentity> {
        self.visible_order
            .iter()
            .copied()
            .filter(|identity| {
                self.instances.get(identity).is_some_and(|record| {
                    surfaces.contains(&record.basis.semantic_surface_identity())
                })
            })
            .collect()
    }

    pub(crate) fn projection_order_snapshot(
        &self,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) -> Option<crate::runtime::persistent_index::UiPersistentOrder<UiMountedInstanceIdentity>>
    {
        (surfaces.len() == self.semantic_surfaces.len()
            && surfaces
                .iter()
                .all(|surface| self.semantic_surfaces.contains_key(surface)))
        .then(|| self.visible_order.clone())
    }

    pub(crate) fn projection_surface(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<(
        super::UiSurfaceBindingIdentityView,
        UiMountedProjectionAudience,
    )> {
        Some((
            self.bindings.get(&surface)?.view,
            self.semantic_surfaces.get(&surface).copied()?,
        ))
    }
}

impl UiMountedProjectionAffectedInstances {
    pub(crate) fn instances(&self) -> &[UiMountedInstanceIdentity] {
        &self.instances
    }

    pub(crate) fn index_entries_touched(&self) -> usize {
        self.index_entries_touched
    }
}

fn next(counter: &AtomicU64) -> Result<u64, UiMountedIdentityDenial> {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)
}

impl UiMountedIdentityFrameCandidate {
    pub(super) fn frame(&self) -> UiMountedFrameIdentity {
        self.receipt_basis.frame()
    }

    pub(super) fn receipt_basis(&self) -> &super::UiMountedNodeReceiptBasis {
        &self.receipt_basis
    }
}

impl UiAuthorityAdmittedMountedFrame {
    fn new(frame: super::UiPreparedMountedFrame) -> Self {
        Self { frame }
    }

    pub(in crate::mounting) fn into_frame(self) -> super::UiPreparedMountedFrame {
        self.frame
    }
}
