//! Governed dependency marks on the existing native envelope and canonical touched-fact stream.
use super::{admission::WorthQueryWorkflowReadDenial as Denial, *};
use crate::domain_computation::primary_graph::{
    application_installation::WorthQueryWorkflowVocabulary,
    live_delivery::{WorthQueryLiveSourcePoll, WorthQueryLiveSubscription},
    output_lineage::invalidation::{
        CommitTouchEntities, CommitTouchInterest, CommitTouchesStop, InvalidationEditAdmission,
    },
    WorthQueryAdvancementPhase, WorthQuerySelectedProductOperation,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowReadBinding, ApplicationWorkflowReadRoute, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{mvcc::CompanionPreflightBudget, publication::PatchStreamPosition};
/// Coarse invalidation of an immutable view. No mark refreshes its retained data or grants authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowHistoryMark {
    Pending,
    Changed,
    Overflow,
    Closed,
}
/// Same native subscription/cursor/overflow/drop custody as the existing live envelope.
/// Fresh disclosure is required to observe every mark; no original grant is retained.
pub struct WorthQueryWorkflowHistorySubscription {
    target: WorthQueryWorkflowReadTarget,
    support: [u8; 32],
    subscription: Option<WorthQueryLiveSubscription>,
    cursor: u64,
    touches: Option<PatchStreamPosition>,
    discontinuous: bool,
}
impl WorthQueryWorkflowHistorySubscription {
    pub fn basis(&self) -> &WorthQueryWorkflowReadBasis {
        self.target.basis()
    }
    /// Releases only subscription custody. This neither closes nor advances a workflow.
    pub fn close(mut self) {
        self.subscription.take();
    }
}
impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    /// Opens after fresh disclosure, retaining a locator and native subscription rather than a permit.
    pub fn subscribe_workflow_history<W, B>(
        &self,
        data: &WorthQuerySelectedProductOperation<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        vocabulary: &WorthQueryWorkflowVocabulary<'_, S, W>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        input: B::Input,
        request: &WorthQueryRequestScope,
        target: &WorthQueryWorkflowReadTarget,
    ) -> Result<WorthQueryWorkflowHistorySubscription, Denial>
    where
        W: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationWorkflowReadBinding<S>,
    {
        let support = self.admit_history_interest::<W, B>(
            data, phase, vocabulary, principal, input, request, target,
        )?;
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(Denial::Phase)?;
        let source = &self.application().primary_provider.live_delivery;
        // Both cursors are native owner positions. A retired exact start position is a discontinuity.
        let subscription = source
            .open(data.product().observation())
            .map_err(|_| Denial::SubscriptionCapacity)?;
        let cursor = subscription.cursor();
        let basis = data.product().relational_basis().observation();
        let touches = match basis.commit_id() {
            Some(commit) => self
                .application()
                .primary_provider
                .graph
                .source_owner
                .invalidation_owner
                .touch_position_at(basis.identity().branch_id(), commit, &execution)
                .map_err(Denial::Work)?,
            None => None,
        };
        Ok(WorthQueryWorkflowHistorySubscription {
            target: target.clone(),
            support,
            subscription: Some(subscription),
            cursor,
            touches,
            discontinuous: touches.is_none(),
        })
    }
    /// Reads delivered marks only after current security admission; refusal leaves all cursor custody intact.
    pub fn poll_workflow_history<W, B>(
        &self,
        data: &WorthQuerySelectedProductOperation<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        vocabulary: &WorthQueryWorkflowVocabulary<'_, S, W>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        input: B::Input,
        request: &WorthQueryRequestScope,
        interest: &mut WorthQueryWorkflowHistorySubscription,
    ) -> Result<WorthQueryWorkflowHistoryMark, Denial>
    where
        W: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationWorkflowReadBinding<S>,
    {
        let support = self.admit_history_interest::<W, B>(
            data,
            phase,
            vocabulary,
            principal,
            input,
            request,
            &interest.target,
        )?;
        if support != interest.support {
            return Err(Denial::PolicyMismatch);
        }
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(Denial::Phase)?;
        execution.consult().map_err(Denial::Work)?;
        let Some(subscription) = interest.subscription.as_ref() else {
            return Ok(WorthQueryWorkflowHistoryMark::Closed);
        };
        let mut next = interest.cursor;
        let mut closed = false;
        match self
            .application()
            .primary_provider
            .live_delivery
            .poll(subscription, interest.cursor)
        {
            WorthQueryLiveSourcePoll::Overflow { .. } => {
                interest.discontinuous = true;
            }
            WorthQueryLiveSourcePoll::Closed => closed = true,
            WorthQueryLiveSourcePoll::Pending => {}
            WorthQueryLiveSourcePoll::Batch(batch) => {
                next = batch
                    .batch()
                    .sequence
                    .checked_add(1)
                    .ok_or(Denial::Corrupt)?;
            }
        }
        if interest.discontinuous {
            return Ok(WorthQueryWorkflowHistoryMark::Overflow);
        }
        let entities = interest.target.dependencies.elements();
        let basis = self.product().relational_basis().observation();
        let batch = self
            .application()
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .touched_commits_after(
                basis.identity().branch_id(),
                interest.touches,
                basis.commit_id(),
                1,
                CommitTouchInterest {
                    entities: CommitTouchEntities::Slice(entities),
                    every_commit: false,
                },
            );
        let batch = match batch {
            Ok(batch) => batch,
            Err(CommitTouchesStop::RequiresReconstruction { .. }) => {
                interest.discontinuous = true;
                return Ok(WorthQueryWorkflowHistoryMark::Overflow);
            }
        };
        interest.touches = batch.next_cursor;
        interest.cursor = next;
        if !batch.commits.is_empty() {
            Ok(WorthQueryWorkflowHistoryMark::Changed)
        } else if closed {
            Ok(WorthQueryWorkflowHistoryMark::Closed)
        } else {
            Ok(WorthQueryWorkflowHistoryMark::Pending)
        }
    }
    fn admit_history_interest<W, B>(
        &self,
        data: &WorthQuerySelectedProductOperation<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        vocabulary: &WorthQueryWorkflowVocabulary<'_, S, W>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        input: B::Input,
        request: &WorthQueryRequestScope,
        target: &WorthQueryWorkflowReadTarget,
    ) -> Result<[u8; 32], Denial>
    where
        W: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationWorkflowReadBinding<S>,
    {
        if !std::ptr::eq(vocabulary.runtime(), self.application()) {
            return Err(Denial::PolicyMismatch);
        }
        let workflow = vocabulary.workflow_spec_for(self);
        let binding = workflow
            .resolve_read_binding::<B>(
                self.application().installed_schema(),
                ApplicationWorkflowReadRoute::Subject,
            )
            .map_err(Denial::Installation)?;
        let profile = self
            .application()
            .runtime
            .application_query_resource_profile();
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: profile.maximum_work().get() as u64,
            maximum_preparation_bytes: profile.maximum_inline_index_bytes().get() as u64,
        });
        target.admit(
            self,
            data,
            phase,
            &binding,
            principal,
            input,
            request,
            &mut admission,
        )?;
        Ok(*workflow.support_identity_bytes())
    }
}
