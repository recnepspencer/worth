use sha2::{Digest, Sha256};

use super::WorthQueryExecutionResourceContract;

pub(super) fn canonical_resource_contract_token(
    contract: &WorthQueryExecutionResourceContract,
) -> String {
    let mut hasher = Sha256::new();
    let atomic = contract.strategies().iter().any(|s| s.envelope().boundary() == worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary::Atomic);
    hash_text(
        &mut hasher,
        if atomic {
            "worth_query_execution_resource_contract_v3"
        } else {
            "worth_query_execution_resource_contract_v2"
        },
    );
    let WorthQueryExecutionResourceContract::Declared { strategies } = contract else {
        hash_text(&mut hasher, "undeclared");
        return format!("{:x}", hasher.finalize());
    };
    hash_text(&mut hasher, "declared");
    hash_u64(&mut hasher, strategies.len() as u64);
    for strategy in strategies {
        hash_text(&mut hasher, strategy.name().as_str());
        hash_text(
            &mut hasher,
            strategy.provider_requirements().provider().as_str(),
        );
        hash_text(
            &mut hasher,
            strategy.provider_requirements().access_product().as_str(),
        );
        hash_text(
            &mut hasher,
            strategy.provider_requirements().allocator().as_str(),
        );
        hash_text(&mut hasher, strategy.envelope().mode().as_str());
        hash_text(
            &mut hasher,
            strategy.envelope().cancellation_safe_point().as_str(),
        );
        hash_text(
            &mut hasher,
            strategy
                .envelope()
                .degradation()
                .map_or("complete", |degradation| degradation.as_str()),
        );
        hash_text(
            &mut hasher,
            strategy.envelope().partial_effect_posture().as_str(),
        );
        hash_text(
            &mut hasher,
            strategy.envelope().yielded_state_posture().as_str(),
        );
        hash_text(
            &mut hasher,
            strategy.envelope().retained_progress_posture().as_str(),
        );
        if atomic {
            hash_text(&mut hasher, strategy.envelope().boundary().as_str());
            hash_u64(
                &mut hasher,
                strategy.envelope().scale_ceilings().iter().count() as u64,
            );
        }
        for (axis, value) in strategy.envelope().scale_ceilings().iter() {
            hash_text(&mut hasher, axis.as_str());
            hash_u64(&mut hasher, value);
        }
        if atomic {
            hash_u64(
                &mut hasher,
                strategy.envelope().resource_ceilings().iter().count() as u64,
            );
        }
        for (dimension, value) in strategy.envelope().resource_ceilings().iter() {
            hash_text(&mut hasher, dimension.as_str());
            hash_u64(&mut hasher, value);
        }
    }
    format!("{:x}", hasher.finalize())
}

fn hash_text(hasher: &mut Sha256, value: &str) {
    hash_u64(hasher, value.len() as u64);
    hasher.update(value.as_bytes());
}

fn hash_u64(hasher: &mut Sha256, value: u64) {
    hasher.update(value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use worth_query_declaration::facade::domain_computation::{
        WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
    };

    use super::super::{
        WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
        WorthQueryExecutionProviderFamily, WorthQueryExecutionProviderRequirements,
        WorthQueryExecutionResourceContract, WorthQueryExecutionResourceEnvelope,
        WorthQueryExecutionStrategyContract, WorthQueryExecutionStrategyName,
    };

    #[test]
    fn length_framing_distinguishes_delimiter_redistribution() {
        let first = contract("a|b", "c", "d", "e");
        let second = contract("a", "b", "c", "d|e");

        assert_ne!(first.canonical_identity(), second.canonical_identity());
    }

    fn contract(
        strategy: &str,
        provider: &str,
        access: &str,
        allocator: &str,
    ) -> WorthQueryExecutionResourceContract {
        WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
            WorthQueryExecutionStrategyName::new(strategy).unwrap(),
            WorthQueryExecutionResourceEnvelope::bounded(
                8,
                8,
                WorthQueryExecutionMode::Synchronous,
                WorthQueryCancellationSafePointFamily::new("chunk").unwrap(),
            ),
            WorthQueryExecutionProviderRequirements::new(
                WorthQueryExecutionProviderFamily::new(provider).unwrap(),
                WorthQueryExecutionAccessProductFamily::new(access).unwrap(),
                WorthQueryExecutionAllocatorFamily::new(allocator).unwrap(),
            ),
        )])
        .unwrap()
    }
}
