use worth_foundational::facade::CanonicalDigestDerivationDenial;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkEvidence, WorthQueryTemporalIntentCandidate,
};

use crate::domain_computation::primary_graph::conditional_operation::canonical_identity::{
    CanonicalIdentityMaterial, WorthQueryTemporalRuntimeBindingIdentity,
};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding;

pub(super) struct WorthQueryPreparedTemporalIdempotency {
    binding: WorthQueryApplicationIdempotencyBinding,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryPreparedTemporalIdempotency {
    pub(super) fn binding(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.binding
    }

    pub(super) fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

pub(super) fn prepare_temporal_idempotency<Clock, Input>(
    runtime_binding: &WorthQueryTemporalRuntimeBindingIdentity,
    candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
) -> Result<WorthQueryPreparedTemporalIdempotency, CanonicalDigestDerivationDenial> {
    let mut key = CanonicalIdentityMaterial::new(
        "worth-query.temporal-operation-idempotency-key",
        "worth-query-temporal-operation-idempotency-key-v1",
    );
    key.digest("runtime-binding", runtime_binding.digest());
    key.text("host-idempotency", candidate.idempotency().as_str());
    let (key, key_work) = key.derive()?;

    let mut intent = CanonicalIdentityMaterial::new(
        "worth-query.temporal-operation-intent",
        "worth-query-temporal-operation-intent-v1",
    );
    intent.text("identity", candidate.identity().as_str());
    intent.unsigned_u64("revision", candidate.revision());
    intent.text("input", candidate.input_identity().as_str());
    let (intent, intent_work) = intent.derive()?;

    Ok(WorthQueryPreparedTemporalIdempotency {
        binding: WorthQueryApplicationIdempotencyBinding::new(*key.bytes(), *intent.bytes()),
        canonical_work: key_work.combine(intent_work),
    })
}
