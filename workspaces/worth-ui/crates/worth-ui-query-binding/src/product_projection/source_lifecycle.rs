use worth_query::facade::runtime;
use worth_runtime_bridge::facade::{
    AdmittedBridgeAsyncRequestIdentity, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest, RuntimeBridge,
};

use crate::{
    UiProjectionConsumptionBudget, UiScalarProjectionBatchOutcome, UiScalarProjectionBinding,
    UiScalarProjectionFactReceipt, UiScalarProjectionRegistration,
};

use super::{SharedSourceState, WorthUiScalarProjectionSourceRecord};

mod declaration;
mod installed_view;
mod product_action;
mod projection_shutdown;
mod revalidation;

pub use product_action::{
    WorthUiScalarProjectionActionAdvance, WorthUiScalarProjectionActionDenied,
    WorthUiScalarProjectionActionExecution, WorthUiScalarProjectionActionIndeterminate,
    WorthUiScalarProjectionActionInstallation, WorthUiScalarProjectionActionLiveOwner,
    WorthUiScalarProjectionActionOutcome, WorthUiScalarProjectionActionPublicationCompletion,
    WorthUiScalarProjectionActionRequest,
};

use declaration::{admitted_completion, async_request, declare_scalar_view, scalar_binding};
use projection_shutdown::WorthUiScalarProjectionShutdownOwners;
use revalidation::revalidate;

type ScalarLiveView = runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape>;

pub struct WorthUiScalarProjectionInstallation {
    registration: UiScalarProjectionRegistration,
    initial: WorthUiScalarProjectionAdvance,
    installed_domain: crate::WorthUiInstalledQueryDomain,
}

pub struct WorthUiScalarProjectionLiveOwner {
    workspace: runtime::WorthQueryWorkspace,
    bridge: RuntimeBridge,
    source: SharedSourceState,
    view: ScalarLiveView,
    binding: UiScalarProjectionBinding,
    request: AdmittedBridgeAsyncRequestIdentity,
    predecessor: UiScalarProjectionFactReceipt,
    revision: u64,
    basis_revision: u64,
}

pub struct WorthUiScalarProjectionAdvance {
    observation: crate::UiProjectionObservation,
    completion: WorthUiScalarProjectionPublicationCompletion,
}

pub struct WorthUiScalarProjectionPublicationCompletion {
    owner: WorthUiScalarProjectionUnpublishedOwner,
    expected_owner_order: u64,
    retained_predecessor: Option<UiScalarProjectionFactReceipt>,
}

struct WorthUiScalarProjectionUnpublishedOwner {
    workspace: runtime::WorthQueryWorkspace,
    bridge: RuntimeBridge,
    source: SharedSourceState,
    view: ScalarLiveView,
    binding: UiScalarProjectionBinding,
    request: AdmittedBridgeAsyncRequestIdentity,
    revision: u64,
    basis_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiScalarProjectionSourceCloseReceipt {
    owner_terminal: bool,
    live_source_count: usize,
    live_attempt_count: usize,
    live_resource_count: usize,
    live_consumer_lease_count: usize,
    retained_projection_count: usize,
    projection_receipt_count: usize,
}

#[derive(Debug)]
pub enum WorthUiScalarProjectionAdvanceError {
    Bridge(String),
    Query(runtime::WorthQueryAsyncSourceBindingError),
    RevisionNotSuccessor { active: u64, submitted: u64 },
    UnexpectedUnchanged,
}

#[derive(Debug)]
pub enum WorthUiScalarProjectionSourceCloseError {
    Query(runtime::WorthQueryRuntimeError),
}

impl WorthUiScalarProjectionInstallation {
    #[allow(
        clippy::result_large_err,
        reason = "cold source opening preserves exact Query installation failure topology"
    )]
    pub(super) fn open(
        mut workspace: runtime::WorthQueryWorkspace,
        bridge: RuntimeBridge,
        source: SharedSourceState,
    ) -> Result<Self, super::WorthUiScalarProjectionInstallationError> {
        let installed_domain = crate::WorthUiQueryHost::from_workspace(&workspace)
            .installed_domain()
            .map_err(|error| {
                super::WorthUiScalarProjectionInstallationError::SourceLifecycle(format!(
                    "Worth UI domain discovery failed: {error:?}"
                ))
            })?;
        let request = async_request(&bridge, 0)
            .map_err(super::WorthUiScalarProjectionInstallationError::SourceLifecycle)?;
        let view = declare_scalar_view(&mut workspace, &request).map_err(|error| {
            super::WorthUiScalarProjectionInstallationError::SourceLifecycle(error.to_string())
        })?;
        let (mut binding, registration) = scalar_binding(&workspace)?;
        let initial = binding
            .consume_initial_async_result(
                &mut workspace,
                &view,
                UiProjectionConsumptionBudget::platform_pulse(),
            )
            .map_err(|error| {
                super::WorthUiScalarProjectionInstallationError::SourceLifecycle(format!(
                    "{error:?}"
                ))
            })?;
        let (fact, retained) = initial.into_fact_and_predecessor();
        debug_assert!(retained.is_none());
        Ok(Self {
            registration,
            installed_domain,
            initial: issue_advance(
                WorthUiScalarProjectionUnpublishedOwner {
                    workspace,
                    bridge,
                    source,
                    view,
                    binding,
                    request,
                    revision: 0,
                    basis_revision: 0,
                },
                fact,
                retained,
            ),
        })
    }

    pub fn into_initial_advance(self) -> WorthUiScalarProjectionAdvance {
        self.initial
    }

    pub fn into_parts(
        self,
    ) -> (
        UiScalarProjectionRegistration,
        WorthUiScalarProjectionAdvance,
    ) {
        (self.registration, self.initial)
    }
}

impl WorthUiScalarProjectionAdvance {
    pub fn observation(&self) -> &crate::UiProjectionObservation {
        &self.observation
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::UiProjectionObservation,
        WorthUiScalarProjectionPublicationCompletion,
    ) {
        (self.observation, self.completion)
    }
}

impl WorthUiScalarProjectionPublicationCompletion {
    #[allow(
        clippy::result_large_err,
        reason = "the error returns the exact affine fact to its lifecycle owner"
    )]
    pub fn admit_publication(
        self,
        predecessor: crate::UiScalarProjectionObservation,
    ) -> Result<WorthUiScalarProjectionLiveOwner, crate::UiScalarProjectionObservation> {
        let predecessor = predecessor.into_fact();
        if predecessor.core().observation_order() != self.expected_owner_order {
            return Err(predecessor.into_observation());
        }
        let owner = self.owner;
        let predecessor = self.retained_predecessor.unwrap_or(predecessor);
        Ok(WorthUiScalarProjectionLiveOwner {
            workspace: owner.workspace,
            bridge: owner.bridge,
            source: owner.source,
            view: owner.view,
            binding: owner.binding,
            request: owner.request,
            predecessor,
            revision: owner.revision,
            basis_revision: owner.basis_revision,
        })
    }
}

impl WorthUiScalarProjectionLiveOwner {
    pub fn advance(
        mut self,
        record: WorthUiScalarProjectionSourceRecord,
    ) -> Result<WorthUiScalarProjectionAdvance, WorthUiScalarProjectionAdvanceError> {
        if record.revision() <= self.revision {
            return Err(WorthUiScalarProjectionAdvanceError::RevisionNotSuccessor {
                active: self.revision,
                submitted: record.revision(),
            });
        }
        let revision = record.revision();
        let payload_bytes = record.status().len().saturating_add(8) as u64;
        self.source.borrow_mut().publish(record);
        let (request, predecessor, basis_revision) = if self.revision == 0 {
            (self.request.clone(), self.predecessor, self.basis_revision)
        } else {
            let basis_revision = revision.max(self.basis_revision.saturating_add(1));
            let (request, predecessor) = revalidate(
                &self.bridge,
                &mut self.workspace,
                &self.view,
                &mut self.binding,
                &self.request,
                self.predecessor,
                basis_revision,
            )
            .map_err(|stop| stop.into_error())?;
            (request, predecessor, basis_revision)
        };

        let completion = admitted_completion(&self.bridge, &request, payload_bytes)?;
        let ordering = self
            .bridge
            .order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
                BridgeMixedCauseOrderingLaneKind::Authoritative,
                vec![BridgeMixedCauseOrderingInput::AsyncCompletion(completion)],
            ));
        let batch = self
            .workspace
            .admit_bridge_async_result_transitions(&self.view, &ordering)
            .map_err(WorthUiScalarProjectionAdvanceError::Query)?;
        let current = self.binding.consume_async_result_batch(
            &mut self.workspace,
            batch,
            Some(predecessor),
            UiProjectionConsumptionBudget::platform_pulse(),
        );
        let UiScalarProjectionBatchOutcome::Advanced(current) = current else {
            return Err(WorthUiScalarProjectionAdvanceError::UnexpectedUnchanged);
        };
        let (fact, retained) = current.into_fact_and_predecessor();
        Ok(issue_advance(
            WorthUiScalarProjectionUnpublishedOwner {
                workspace: self.workspace,
                bridge: self.bridge,
                source: self.source,
                view: self.view,
                binding: self.binding,
                request,
                revision,
                basis_revision,
            },
            fact,
            retained,
        ))
    }

    #[allow(
        clippy::result_large_err,
        reason = "shutdown is cold and preserves Query's exact terminal denial"
    )]
    pub fn close(
        self,
    ) -> Result<WorthUiScalarProjectionSourceCloseReceipt, WorthUiScalarProjectionSourceCloseError>
    {
        let Self {
            mut workspace,
            bridge,
            source,
            view,
            binding,
            request,
            predecessor,
            revision: _,
            basis_revision: _,
        } = self;
        let query = workspace
            .close_bridge_async_live_view(view)
            .map_err(WorthUiScalarProjectionSourceCloseError::Query)?;
        let projection = WorthUiScalarProjectionShutdownOwners::new(binding, predecessor).release();
        drop((bridge, request));
        drop(workspace);
        let live_source_count = source.borrow().live_source_count();
        Ok(WorthUiScalarProjectionSourceCloseReceipt {
            owner_terminal: query.lane_terminal()
                && live_source_count == 0
                && query.live_async_attempt_count() == 0
                && query.live_resource_count() == 0
                && query.live_consumer_lease_count() == 0
                && projection.retained_projection_count() == 0
                && projection.projection_receipt_count() == 0,
            live_source_count,
            live_attempt_count: query.live_async_attempt_count(),
            live_resource_count: query.live_resource_count(),
            live_consumer_lease_count: query.live_consumer_lease_count(),
            retained_projection_count: projection.retained_projection_count(),
            projection_receipt_count: projection.projection_receipt_count(),
        })
    }
}

impl WorthUiScalarProjectionSourceCloseReceipt {
    pub fn owner_terminal(self) -> bool {
        self.owner_terminal
    }

    pub fn live_source_count(self) -> usize {
        self.live_source_count
    }

    pub fn live_attempt_count(self) -> usize {
        self.live_attempt_count
    }

    pub fn live_resource_count(self) -> usize {
        self.live_resource_count
    }

    pub fn live_consumer_lease_count(self) -> usize {
        self.live_consumer_lease_count
    }

    pub fn retained_projection_count(self) -> usize {
        self.retained_projection_count
    }

    pub fn projection_receipt_count(self) -> usize {
        self.projection_receipt_count
    }
}

fn issue_advance(
    owner: WorthUiScalarProjectionUnpublishedOwner,
    fact: UiScalarProjectionFactReceipt,
    retained_predecessor: Option<UiScalarProjectionFactReceipt>,
) -> WorthUiScalarProjectionAdvance {
    let expected_owner_order = fact.core().observation_order();
    WorthUiScalarProjectionAdvance {
        observation: crate::UiProjectionObservation::Scalar(fact.into_observation()),
        completion: WorthUiScalarProjectionPublicationCompletion {
            owner,
            expected_owner_order,
            retained_predecessor,
        },
    }
}
