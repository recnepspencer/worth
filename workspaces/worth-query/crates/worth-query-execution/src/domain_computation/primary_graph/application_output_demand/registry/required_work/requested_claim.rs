//! Exact temporary custody between a failed native read and caller publication.
use super::super::{
    DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint,
    WorthQueryOutputDemandInterest, WorthQueryOutputDemandNotifications,
};
use super::selection::{charge_required_key_lookup, work_denial, SelectedReadyReadmission};
use super::*;

/// Each node funds its own Box and interest key. No source is reconstructed.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct RequestedOutputReadClaim
{
    node: Box<RequestedReadNode>,
    // Refund after the interest key and physical node backing leave custody.
    _capacity: super::super::required_custody::RequiredOutputCustodyCapacity,
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Option<(SelectedReadyReadmission, RequestedOutputReadClaim)>,
        WorthQueryOutputDemandDenial,
    > {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(key, admission)?;
        charge_required_key_lookup(&state, key, admission)?;
        let Some(record) = state.records.get(key) else {
            return Ok(None);
        };
        let DemandState::Output(output) = &record.state else {
            return Ok(None);
        };
        if !matches!(output.advancement, WorthQueryOutputAdvancement::Idle) {
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
        let key_bytes = std::mem::size_of::<RequestedReadNode>()
            .checked_add(key.producer.len())
            .ok_or_else(capacity_denial)?;
        admission
            .charge_external_work(key.producer.len() as u64 * 2 + 16)
            .map_err(|_| work_denial())?;
        admission
            .admit_read_scratch(key_bytes as u64)
            .map_err(|_| capacity_denial())?;
        let selected = SelectedReadyReadmission {
            completion: completion.clone(),
            readmission: Arc::clone(source),
            membership: Arc::clone(membership),
        };
        let wake = Arc::clone(&record.wake);
        let member = state.prepare_required_member(key)?;
        let member_bytes = member.as_ref().map_or(0, |member| member.reserved_bytes());
        let total = state
            .required_reserved_bytes
            .checked_add(member_bytes)
            .and_then(|bytes| bytes.checked_add(key_bytes))
            .ok_or_else(capacity_denial)?;
        if !state.has_required_capacity(total) {
            return Err(super::super::required_custody::full_custody_denial(
                key_bytes + member_bytes,
                state.required_budget_bytes,
            ));
        }
        let capacity = state.reserve_required_custody_capacity(key_bytes)?;
        let interest = WorthQueryOutputDemandInterest {
            key: key.clone(),
            requires_output: true,
            notifications: WorthQueryOutputDemandNotifications { wake },
            owner: self.clone(),
        };
        let record = state
            .records
            .get_mut(key)
            .expect("selected cached Ready retained");
        record.interests = interests;
        record.required_interests = required;
        state.install_required_member(member);
        drop(state);
        Ok(Some((
            selected,
            RequestedOutputReadClaim {
                node: Box::new(RequestedReadNode {
                    _interest: interest,
                    next: None,
                }),
                _capacity: capacity,
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
