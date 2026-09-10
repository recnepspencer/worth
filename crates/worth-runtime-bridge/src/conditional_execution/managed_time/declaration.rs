use std::{num::NonZeroUsize, sync::Arc};

use super::contract::{validate_identity, BridgeManagedClockLease};
use super::{
    BridgeManagedClockBinding, BridgeManagedClockInstallationParts, BridgeManagedClockLane,
    BridgeManagedTemporalDenial, BridgeManagedTemporalDenialKind,
};

/// Complete pre-seal declaration. Signal temporal custody is issued only after
/// the conditional graph has entered its one-owner service posture.
pub(in crate::conditional_execution) struct BridgeManagedClockDeclaration {
    runtime_key: u64,
    identity: Arc<str>,
    source: Arc<str>,
    timeline: Arc<str>,
    lowering: Arc<super::super::BridgeInstalledConditionalLowering>,
    lease: Arc<BridgeManagedClockLease>,
    maximum_active_intents: usize,
    maximum_due: NonZeroUsize,
    retention: Arc<super::super::retention::BridgeRetentionLedger>,
    reservation: super::super::retention::BridgeRetentionReservation,
}

impl BridgeManagedClockDeclaration {
    pub(in crate::conditional_execution) fn admit(
        runtime_key: u64,
        retention: &Arc<super::super::retention::BridgeRetentionLedger>,
        parts: BridgeManagedClockInstallationParts<'_>,
    ) -> Result<Self, BridgeManagedTemporalDenial> {
        validate_identity(&parts.binding_identity, "managed clock binding")?;
        validate_identity(&parts.source_identity, "managed clock source")?;
        validate_identity(&parts.timeline_identity, "managed clock timeline")?;
        let maximum_due = NonZeroUsize::new(parts.maximum_due_wakes_per_observation)
            .ok_or_else(|| invalid("managed clock due-wake bound must be non-zero"))?;
        if parts.maximum_active_intents == 0 {
            return Err(invalid(
                "managed clock active-intent capacity must be non-zero",
            ));
        }
        let reservation = super::retention::reserve_clock(retention, parts.maximum_active_intents)?;
        let binding_reservation = super::retention::reserve_binding(retention)?;
        Ok(Self {
            retention: Arc::clone(retention),
            reservation,
            runtime_key,
            identity: parts.binding_identity,
            source: parts.source_identity,
            timeline: parts.timeline_identity,
            lowering: Arc::clone(parts.lowering),
            lease: Arc::new(BridgeManagedClockLease::issue(binding_reservation)),
            maximum_active_intents: parts.maximum_active_intents,
            maximum_due,
        })
    }

    pub(in crate::conditional_execution) fn identity(&self) -> &Arc<str> {
        &self.identity
    }

    pub(in crate::conditional_execution) fn binding(&self) -> BridgeManagedClockBinding {
        BridgeManagedClockBinding {
            bridge_runtime_key: self.runtime_key,
            binding_identity: Arc::clone(&self.identity),
            source_identity: Arc::clone(&self.source),
            timeline_identity: Arc::clone(&self.timeline),
            lease: Arc::clone(&self.lease),
        }
    }

    pub(in crate::conditional_execution) fn seal(
        self,
    ) -> Result<BridgeManagedClockLane, BridgeManagedTemporalDenial> {
        let port = self.lowering.signal_port().ok_or_else(|| {
            invalid("managed clock declaration requires its sealed Signal service")
        })?;
        let temporal = port
            .admit_temporal_partition(
                NonZeroUsize::new(self.maximum_active_intents)
                    .expect("admitted positive intent bound"),
            )
            .map_err(|denial| {
                BridgeManagedTemporalDenial::new(
                    BridgeManagedTemporalDenialKind::SignalTemporalFailure,
                    format!("Signal temporal partition admission was denied: {denial:?}"),
                )
            })?;
        Ok(BridgeManagedClockLane::new(
            self.lowering,
            self.source,
            self.timeline,
            self.lease,
            self.maximum_active_intents,
            self.maximum_due,
            temporal,
            self.retention,
            self.reservation,
        ))
    }
}

fn invalid(detail: &str) -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(BridgeManagedTemporalDenialKind::InvalidContract, detail)
}
