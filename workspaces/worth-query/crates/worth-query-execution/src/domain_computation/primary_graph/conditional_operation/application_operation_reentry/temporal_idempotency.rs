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
pub(in crate::domain_computation::primary_graph::conditional_operation) enum TemporalIdempotencyScope<'a> {
    ProcessLocal,
    Home {
        owner: worth_foundational::facade::DurableWorldOwnerIdentity,
        branch: &'a str,
        incarnation: u64,
    },
}

fn derive_temporal_idempotency_parts(
    runtime_binding: &WorthQueryTemporalRuntimeBindingIdentity,
    scope: TemporalIdempotencyScope<'_>,
    host_idempotency: &str,
    intent_identity: &str,
    revision: u64,
    input_identity: &str,
) -> Result<WorthQueryPreparedTemporalIdempotency, CanonicalDigestDerivationDenial> {
    let mut key = CanonicalIdentityMaterial::new(
        "worth-query.temporal-operation-idempotency-key",
        "worth-query-temporal-operation-idempotency-key-v2",
    );
    match scope {
        TemporalIdempotencyScope::ProcessLocal => {
            key.text("scope", "process-local");
            key.digest("runtime-binding", runtime_binding.digest());
        }
        TemporalIdempotencyScope::Home {
            owner,
            branch,
            incarnation,
        } => {
            key.text("scope", "home");
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
            key.digest("definition", runtime_binding.definition());
        }
    }
    key.text("host-idempotency", host_idempotency);
    let (key, key_work) = key.derive()?;

    let mut intent = CanonicalIdentityMaterial::new(
        "worth-query.temporal-operation-intent",
        "worth-query-temporal-operation-intent-v1",
    );
    intent.text("identity", intent_identity);
    intent.unsigned_u64("revision", revision);
    intent.text("input", input_identity);
    let (intent, intent_work) = intent.derive()?;

    Ok(WorthQueryPreparedTemporalIdempotency {
        binding: WorthQueryApplicationIdempotencyBinding::new(*key.bytes(), *intent.bytes()),
        canonical_work: key_work.combine(intent_work),
    })
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn derive_for_product(
    runtime_binding: &WorthQueryTemporalRuntimeBindingIdentity,
    product: &crate::basis::WorthQueryProductBranchLease,
    intent_identity: &str,
    revision: u64,
    host_idempotency: &str,
    input_identity: &str,
) -> Result<([u8; 32], [u8; 32]), CanonicalDigestDerivationDenial> {
    let selected = product.observation();
    let scope = match selected.owner_identity().durable() {
        Some(owner) => TemporalIdempotencyScope::Home {
            owner,
            branch: selected.branch_identity().name().as_str(),
            incarnation: selected.lifecycle_incarnation().ordinal(),
        },
        None => TemporalIdempotencyScope::ProcessLocal,
    };
    derive_temporal_idempotency_parts(
        runtime_binding,
        scope,
        host_idempotency,
        intent_identity,
        revision,
        input_identity,
    )
    .map(|prepared| {
        (
            *prepared.binding().key_identity(),
            *prepared.binding().intent_identity(),
        )
    })
}

pub(super) fn prepare_temporal_idempotency<Clock, Input>(
    runtime_binding: &WorthQueryTemporalRuntimeBindingIdentity,
    candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
    scope: TemporalIdempotencyScope<'_>,
) -> Result<WorthQueryPreparedTemporalIdempotency, CanonicalDigestDerivationDenial> {
    derive_temporal_idempotency_parts(
        runtime_binding,
        scope,
        candidate.idempotency().as_str(),
        candidate.identity().as_str(),
        candidate.revision(),
        candidate.input_identity().as_str(),
    )
}
