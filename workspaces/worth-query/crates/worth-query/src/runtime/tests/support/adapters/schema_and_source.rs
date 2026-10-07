use super::*;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::{
    WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQuerySnapshotIdentity,
};
use crate::runtime::backend::WorthQueryRuntimeSnapshotIdentityAdapter;

impl WorthQueryRuntimeSchemaAdapter for TestSchemaAdapter {
    fn admit_live_view(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        _schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_live_view_declaration_admission_receipt(name, request);
        Ok(self.build_live_view_declaration_boundary_receipt(name, request, receipt))
    }
}

pub(in crate::runtime::tests) struct TestSchemaAdapter;

pub(in crate::runtime::tests) struct DenyingSchemaAdapter;

impl WorthQueryRuntimeSchemaAdapter for DenyingSchemaAdapter {
    fn admit_live_view(
        &self,
        _name: &str,
        _request: &DeclarativeLiveQueryRequest,
        _schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "schema admission denied by test adapter",
        ))
    }
}

pub(in crate::runtime::tests) struct DriftingSchemaReceiptAdapter;

impl WorthQueryRuntimeSchemaAdapter for DriftingSchemaReceiptAdapter {
    fn admit_live_view(
        &self,
        _name: &str,
        request: &DeclarativeLiveQueryRequest,
        _schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_live_view_declaration_admission_receipt("drifted.view", request);
        Ok(self.build_live_view_declaration_boundary_receipt("drifted.view", request, receipt))
    }
}

#[derive(Default)]
pub(in crate::runtime::tests) struct TestSourceAdapter {
    live_views: BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryMutationTargetCollectionIdentity>,
    fail_declare: bool,
    fail_close: bool,
}

impl TestSourceAdapter {
    pub(in crate::runtime::tests) fn fail_declare() -> Self {
        Self {
            live_views: BTreeMap::new(),
            fail_declare: true,
            fail_close: false,
        }
    }

    pub(in crate::runtime::tests) fn fail_close() -> Self {
        Self {
            live_views: BTreeMap::new(),
            fail_declare: false,
            fail_close: true,
        }
    }
}

impl WorthQueryRuntimeSourceAdapter for TestSourceAdapter {
    fn declare_live_view(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        if self.fail_declare {
            return Err(WorthQueryWorkspaceError::new(
                "source declaration denied by test adapter",
            ));
        }
        let live_target = WorthQueryLiveArtifactTarget::from_view_name(name.clone());
        self.live_views
            .insert(live_target, request.target_collection_identity());
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        if self.fail_close {
            return Err(WorthQueryWorkspaceError::new(
                "source close denied by test adapter",
            ));
        }
        self.live_views
            .remove(&WorthQueryLiveArtifactTarget::from_view_name(name));
        Ok(())
    }

    fn live_entities_for_target(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, crate::memory_workspace::WorthQueryWorkspaceError> {
        Ok(Vec::new())
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        let mut affected = receipt
            .deltas
            .iter()
            .flat_map(|delta| {
                self.live_views
                    .iter()
                    .filter(move |(_, collection)| {
                        delta
                            .target_collection_identity()
                            .same_target_collection_as(collection)
                    })
                    .map(|(target, _)| target.clone())
            })
            .collect::<Vec<_>>();
        affected.sort();
        affected.dedup();
        affected
    }
}

pub(in crate::runtime::tests) struct CountingSourceAdapter {
    pub(in crate::runtime::tests) declared_live_views: std::rc::Rc<std::cell::Cell<usize>>,
    closed_live_views: Option<std::rc::Rc<std::cell::Cell<usize>>>,
    close_denied: Option<std::rc::Rc<std::cell::Cell<bool>>>,
    inner: TestSourceAdapter,
}

impl CountingSourceAdapter {
    pub(in crate::runtime::tests) fn new(
        declared_live_views: std::rc::Rc<std::cell::Cell<usize>>,
    ) -> Self {
        Self {
            declared_live_views,
            closed_live_views: None,
            close_denied: None,
            inner: TestSourceAdapter::default(),
        }
    }

    pub(in crate::runtime::tests) fn lifecycle_counting(
        declared_live_views: std::rc::Rc<std::cell::Cell<usize>>,
        closed_live_views: std::rc::Rc<std::cell::Cell<usize>>,
        close_denied: std::rc::Rc<std::cell::Cell<bool>>,
    ) -> Self {
        Self {
            declared_live_views,
            closed_live_views: Some(closed_live_views),
            close_denied: Some(close_denied),
            inner: TestSourceAdapter::default(),
        }
    }
}

impl WorthQueryRuntimeSourceAdapter for CountingSourceAdapter {
    fn declare_live_view(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        self.declared_live_views
            .set(self.declared_live_views.get().saturating_add(1));
        self.inner.declare_live_view(name, request, schema_view)
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        if self
            .close_denied
            .as_ref()
            .is_some_and(|denied| denied.get())
        {
            return Err(WorthQueryWorkspaceError::new(
                "source close denied by lifecycle counting adapter",
            ));
        }
        if let Some(closed_live_views) = self.closed_live_views.as_ref() {
            closed_live_views.set(closed_live_views.get().saturating_add(1));
        }
        self.inner.close_live_view(name)
    }

    fn live_entities_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, crate::memory_workspace::WorthQueryWorkspaceError> {
        self.inner.live_entities_for_target(target)
    }

    fn drain_live_patches_for_target(
        &mut self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        self.inner.drain_live_patches_for_target(target)
    }

    fn affected_live_view_targets(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        self.inner.affected_live_view_targets(receipt)
    }
}

#[derive(Default)]
pub(in crate::runtime::tests) struct TestSnapshotIdentityAdapter;

impl WorthQueryRuntimeSnapshotIdentityAdapter for TestSnapshotIdentityAdapter {
    fn current_snapshot_identity(
        &self,
    ) -> Result<
        WorthQuerySnapshotIdentity,
        worth_query_execution::facade::primary_graph::WorthQueryHandleDenial,
    > {
        Ok({
            WorthQuerySnapshotIdentity::preview(
                WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimeStateSnapshot)
                    .field_shape(
                        WorthQueryEvidenceTag::new("test_snapshot_authority"),
                        "stable",
                    )
                    .field_usize(WorthQueryEvidenceTag::new("test_snapshot_sequence"), 1)
                    .seal(),
            )
        })
    }
}

#[derive(Default)]
pub(in crate::runtime::tests) struct DriftingSnapshotIdentityAdapter {
    snapshot_sequence: std::cell::Cell<u64>,
}

impl WorthQueryRuntimeSnapshotIdentityAdapter for DriftingSnapshotIdentityAdapter {
    fn current_snapshot_identity(
        &self,
    ) -> Result<
        WorthQuerySnapshotIdentity,
        worth_query_execution::facade::primary_graph::WorthQueryHandleDenial,
    > {
        Ok({
            let snapshot_sequence = self.snapshot_sequence.get().saturating_add(1);
            self.snapshot_sequence.set(snapshot_sequence);
            WorthQuerySnapshotIdentity::preview(
                WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimeStateSnapshot)
                    .field_usize(
                        WorthQueryEvidenceTag::new("drifting_snapshot_sequence"),
                        snapshot_sequence as usize,
                    )
                    .seal(),
            )
        })
    }
}
