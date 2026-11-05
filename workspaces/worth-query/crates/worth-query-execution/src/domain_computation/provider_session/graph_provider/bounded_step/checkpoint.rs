use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::provider_anchor::WorthQueryGraphProviderAnchor;
use super::{
    WorthQueryCooperativeGraphProviderExecution, WorthQueryGraphProviderExecution,
    WorthQueryProviderCheckpointReleaseDisposition, WorthQueryProviderCheckpointReleaseEvidence,
    WorthQueryProviderCheckpointRestoreInvocation, WorthQueryProviderCheckpointRetentionFailure,
};
use crate::execution_digest::hash_parts;
use {
    crate::domain_computation::vocabulary::provider_session::failure::WorthQueryGraphProviderFailure,
    crate::domain_computation::WorthQueryGraphProviderCall,
};

static NEXT_PROVIDER_CHECKPOINT: AtomicU64 = AtomicU64::new(1);

pub trait WorthQueryGraphProviderCheckpoint: Send + 'static {
    fn retained_bytes(&self) -> u64;

    fn restore(
        &self,
        call: &WorthQueryGraphProviderCall,
        memory: &mut super::WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    >;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderCheckpointEvidence {
    identity: Arc<str>,
    provider_generation: u64,
    retained_bytes: u64,
}

impl WorthQueryProviderCheckpointEvidence {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn provider_generation(&self) -> u64 {
        self.provider_generation
    }

    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}

pub(crate) struct WorthQueryRetainedGraphProviderCheckpoint {
    evidence: WorthQueryProviderCheckpointEvidence,
    anchor: Arc<WorthQueryGraphProviderAnchor>,
    checkpoint: Option<Box<dyn WorthQueryGraphProviderCheckpoint>>,
}

impl WorthQueryRetainedGraphProviderCheckpoint {
    pub(crate) fn retain(
        anchor: Arc<WorthQueryGraphProviderAnchor>,
        call: &WorthQueryGraphProviderCall,
        checkpoint: Box<dyn WorthQueryGraphProviderCheckpoint>,
    ) -> Result<Self, WorthQueryProviderCheckpointRetentionFailure> {
        let ordinal = NEXT_PROVIDER_CHECKPOINT.fetch_add(1, Ordering::Relaxed);
        let identity = Arc::from(hash_parts(&[
            "worth_query_provider_checkpoint_v1".into(),
            format!("ordinal:{ordinal}"),
            format!("provider-generation:{}", anchor.provider_generation()),
            format!("call:{}", call.call_identity()),
            format!("session:{}", call.provider_session_identity()),
        ]));
        let retained_bytes = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            checkpoint.retained_bytes()
        })) {
            Ok(retained_bytes) => retained_bytes,
            Err(_) => {
                let release_disposition =
                    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(checkpoint)))
                        .is_ok()
                    {
                        WorthQueryProviderCheckpointReleaseDisposition::Released
                    } else {
                        WorthQueryProviderCheckpointReleaseDisposition::Panicked
                    };
                return Err(
                    WorthQueryProviderCheckpointRetentionFailure::retained_byte_probe_panicked(
                        identity,
                        anchor.provider_generation(),
                        release_disposition,
                    ),
                );
            }
        };
        let evidence = WorthQueryProviderCheckpointEvidence {
            identity,
            provider_generation: anchor.provider_generation(),
            retained_bytes,
        };
        Ok(Self {
            evidence,
            anchor,
            checkpoint: Some(checkpoint),
        })
    }

    pub(crate) fn evidence(&self) -> &WorthQueryProviderCheckpointEvidence {
        &self.evidence
    }

    pub(crate) fn provider_generation_matches_anchor(&self) -> bool {
        self.evidence.provider_generation() == self.anchor.provider_generation()
    }

    pub(crate) fn provider_anchor(&self) -> Arc<WorthQueryGraphProviderAnchor> {
        Arc::clone(&self.anchor)
    }

    pub(crate) fn invoke_restore(
        &self,
        call: &WorthQueryGraphProviderCall,
        memory: &mut super::WorthQueryGraphProviderRestoreMemory,
    ) -> WorthQueryProviderCheckpointRestoreInvocation {
        let checkpoint = self
            .checkpoint
            .as_ref()
            .expect("retained checkpoint remains present until explicit release");
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            checkpoint.restore(call, memory)
        })) {
            Ok(result) => WorthQueryProviderCheckpointRestoreInvocation::Returned(
                result.and_then(|admitted| memory.validate_returned_execution(admitted)),
            ),
            Err(_) => WorthQueryProviderCheckpointRestoreInvocation::Panicked,
        }
    }

    pub(crate) fn release(mut self) -> WorthQueryProviderCheckpointReleaseEvidence {
        let evidence = self.evidence.clone();
        let checkpoint = self
            .checkpoint
            .take()
            .expect("retained provider checkpoint can be released once");
        let disposition = if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drop(checkpoint)
        }))
        .is_ok()
        {
            WorthQueryProviderCheckpointReleaseDisposition::Released
        } else {
            WorthQueryProviderCheckpointReleaseDisposition::Panicked
        };
        WorthQueryProviderCheckpointReleaseEvidence::new(evidence, disposition)
    }
}

impl Drop for WorthQueryRetainedGraphProviderCheckpoint {
    fn drop(&mut self) {
        let Some(checkpoint) = self.checkpoint.take() else {
            return;
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(checkpoint)));
    }
}
