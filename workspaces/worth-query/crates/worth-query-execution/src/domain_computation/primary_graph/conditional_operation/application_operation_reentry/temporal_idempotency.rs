use worth_foundational::facade::CanonicalDigestDerivationDenial;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkEvidence, WorthQueryTemporalIntentCandidate,
};

use crate::domain_computation::primary_graph::conditional_operation::canonical_identity::{
    CanonicalIdentityMaterial, WorthQueryTemporalRuntimeBindingIdentity,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

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

/// Publication affinity DATA. A live Home incarnation is deliberately absent.
#[derive(Clone, Copy)]
pub(super) enum TemporalIdempotencyScope<'a> {
    ProcessLocal,
    Home {
        owner: worth_foundational::facade::DurableWorldOwnerIdentity,
        branch: &'a str,
        incarnation: u64,
    },
}

pub(super) fn prepare_temporal_idempotency<Clock, Input>(
    runtime_binding: &WorthQueryTemporalRuntimeBindingIdentity,
    candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
    scope: TemporalIdempotencyScope<'_>,
) -> Result<WorthQueryPreparedTemporalIdempotency, CanonicalDigestDerivationDenial> {
    let mut definition_work = WorthQueryCanonicalWorkEvidence::zero();
    let mut key = CanonicalIdentityMaterial::new(
        "worth-query.temporal-operation-idempotency-key",
        "worth-query-temporal-operation-idempotency-key-v3",
    );
    // The two scopes bind disjoint named loci, so their keys never coincide
    // without a separate scope entry.
    match scope {
        TemporalIdempotencyScope::ProcessLocal => {
            key.digest("runtime-binding", runtime_binding.digest());
        }
        TemporalIdempotencyScope::Home {
            owner,
            branch,
            incarnation,
        } => {
            let bytes = owner.home().bytes();
            key.unsigned_u64(
                "home-low",
                u64::from_le_bytes(bytes[..8].try_into().expect("fixed Home identity")),
            );
            key.unsigned_u64(
                "home-high",
                u64::from_le_bytes(bytes[8..].try_into().expect("fixed Home identity")),
            );
            key.unsigned_u64("world-slot", owner.slot().get());
            key.text("branch", branch);
            key.unsigned_u64("branch-incarnation", incarnation);
            let (definition, work) = runtime_binding.definition().derive()?;
            definition_work = work;
            key.digest("definition", definition);
        }
    }
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
        canonical_work: key_work.combine(intent_work).combine(definition_work),
    })
}
