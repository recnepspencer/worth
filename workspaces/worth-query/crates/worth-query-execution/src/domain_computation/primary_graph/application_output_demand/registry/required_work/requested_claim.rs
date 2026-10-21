//! Exact temporary custody between a failed native read and caller publication.
use super::super::DemandState;
use super::super::WorthQueryOutputAdvancement;
use super::super::WorthQueryOutputCheckpoint;
use super::super::WorthQueryOutputDemandInterest;
use super::super::WorthQueryOutputDemandNotifications;
use super::selection::SelectedReadyReadmission;
use super::*;

/// Each node funds its own Box and interest key. No source is reconstructed.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct RequestedOutputReadClaim
{
    node: Box<RequestedReadNode>,
    // Refund after the interest key and physical node backing leave custody.
}

struct RequestedReadNode {
    _interest: WorthQueryOutputDemandInterest,
    next: Option<RequestedOutputReadClaim>,
}

/// Caller-owned claims survive disclosure retry and Pending. Iterative release
/// avoids a recursive drop proportional to the number of outputs actually read.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct RequestedOutputReadClaims {
    head: Option<RequestedOutputReadClaim>,
}

impl RequestedOutputReadClaims {
    pub(super) fn retain(&mut self, mut claim: RequestedOutputReadClaim) {
        claim.node.next = self.head.take();
        self.head = Some(claim);
    }

    /// Transfer already funded nodes into the typed successor that continues
    /// their caller. No backing or source is duplicated during this handoff.
    pub(in crate::domain_computation::primary_graph) fn absorb(&mut self, other: &mut Self) {
        while let Some(mut claim) = other.head.take() {
            other.head = claim.node.next.take();
            self.retain(claim);
        }
    }

    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub(in crate::domain_computation::primary_graph) fn clear(&mut self) {
        while let Some(mut claim) = self.head.take() {
            self.head = claim.node.next.take();
            drop(claim);
        }
    }
}

impl Drop for RequestedOutputReadClaims {
    fn drop(&mut self) {
        self.clear();
    }
}

impl WorthQueryOutputDemandRegistry {
    /// The issuer's exact settlement already joined `key` on this native basis.
    /// Claim only an idle Ready with its installed source, entry and membership.
    /// The normal actor and installed executor still prove currentness afterward.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn claim_cached_requested_ready(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<
        Option<(SelectedReadyReadmission, RequestedOutputReadClaim)>,
        WorthQueryOutputDemandDenial,
    > {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(record) = state.records.get(key) else {
            return Ok(None);
        };
        let DemandState::Output(output) = &record.state else {
            return Ok(None);
        };
        if !matches!(output.advancement(), WorthQueryOutputAdvancement::Idle) {
            return Ok(None);
        }
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else {
            return Ok(None);
        };
        let (Some(source), Some(membership)) =
            (&record.readmission_source, &record.work_membership)
        else {
            return Ok(None);
        };
        if source.producer.get().is_none() {
            return Ok(None);
        }
        let interests = record
            .interests
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        let required = record
            .required_interests
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        let selected = SelectedReadyReadmission {
            completion: completion.clone(),
            readmission: Arc::clone(source),
            membership: Arc::clone(membership),
        };
        let wake = Arc::clone(&record.wake);
        let member = state.prepare_required_member(key)?;
        let interest = WorthQueryOutputDemandInterest {
            key: key.clone(),
            requires_output: true,
            notifications: WorthQueryOutputDemandNotifications { wake },
            owner: self.clone(),
        };
        let mut record = state
            .records
            .get_mut(key)
            .expect("selected cached Ready retained");
        record.interests = interests;
        record.required_interests = required;
        record.finish();
        state.install_required_member(member);
        drop(state);
        Ok(Some((
            selected,
            RequestedOutputReadClaim {
                node: Box::new(RequestedReadNode {
                    _interest: interest,
                    next: None,
                }),
            },
        )))
    }
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "exact requested output claim exceeds required custody",
    )
}
