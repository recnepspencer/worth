//! Funded weak payload custody from the actual completed publication owner.
use super::super::opaque_custody::FundedSourceMaterialReceipt;
use super::{admission, ManagedSourceImpactKey, PreparedSourceImpactHistory};
use crate::mvcc::ManagedSourceHistoryOwner;
use crate::validation::candidate_source::{
    ManagedIndexSourceAvailability, ManagedIndexSourcePreparationDenial as Denial,
    NativeManagedSourceInterpretation,
};
use worth_execution::{ChargedBytes, ExecutionOrderedMap, ExecutionRequest};

#[derive(Clone)]
pub(crate) enum HistoricalSourcePair {
    Conservative,
    Complete {
        before: FundedSourceMaterialReceipt,
        after: FundedSourceMaterialReceipt,
    },
}
impl ChargedBytes for HistoricalSourcePair {
    fn additional_charged_bytes(&self) -> u64 {
        0 // Each weak receipt carries its sealed allocation funding.
    }
}
pub(crate) struct RetainedSourceHistoryMaterial {
    pub(crate) owner: Option<ManagedSourceHistoryOwner>,
    pub(crate) scopes: ExecutionOrderedMap<ManagedSourceImpactKey, HistoricalSourcePair>,
}

impl RetainedSourceHistoryMaterial {
    pub(crate) fn capture(
        execution: ExecutionRequest<'_, '_>,
        interpretation: &NativeManagedSourceInterpretation,
        packet: &PreparedSourceImpactHistory,
    ) -> Result<Self, Denial> {
        let prepared = interpretation
            .prepared
            .get()
            .ok_or(Denial::IncompletePreparation)?;
        let image = interpretation
            .physical_basis()
            .inner
            .root
            .managed_source_image
            .get();
        let mut result = Self {
            owner: None,
            scopes: ExecutionOrderedMap::default(),
        };
        super::super::applied::stage::prepare(execution, |budget| {
            admission::work(budget, 1)?;
            result.owner = image.and_then(|image| image.history_owner());
            // Unsupported/cold native cell custody cannot prove irrelevance.
            if result.owner.is_none() {
                return Ok(());
            }
            // This is the authoritative completed Source preparation roster,
            // not a per-impact search or reconstruction of historical rows.
            for program in &prepared.programs {
                admission::work(
                    budget,
                    packet.catalog.len().checked_add(1).ok_or_else(|| {
                        admission::storage(
                            crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                        )
                    })?,
                )?;
                let slot = packet
                    .catalog
                    .slot(&program.program)
                    .map_err(|_| Denial::ForeignSource)?;
                for root in &program.roots {
                    admission::work(budget, 1)?;
                    if root.program != program.program {
                        return Err(Denial::ForeignSource);
                    }
                    let key = ManagedSourceImpactKey {
                        program: slot,
                        scope: root.scope,
                    };
                    admission::ordered(budget, packet.impacts.len(), size_of_val(&key))?;
                    if !packet.impacts.contains_key(&key) {
                        continue;
                    }
                    let pair = match (&root.committed, &root.proposed, root.availability) {
                        (Some(before), Some(after), ManagedIndexSourceAvailability::Available) => {
                            // Eligibility of BOTH actual prepared sides was
                            // established before consulting the selected cache.
                            let before = match image
                                .ok_or(Denial::ForeignSource)?
                                .history_payload_receipt(
                                    &execution,
                                    &program.program,
                                    root.scope,
                                    budget,
                                )? {
                                Some(receipt) => receipt,
                                None => FundedSourceMaterialReceipt::retain(&execution, before)
                                    .map_err(admission::storage)?,
                            };
                            let after = FundedSourceMaterialReceipt::retain(&execution, after)
                                .map_err(admission::storage)?;
                            HistoricalSourcePair::Complete { before, after }
                        }
                        _ => HistoricalSourcePair::Conservative,
                    };
                    // A missing prepared side is retained explicitly and must
                    // never fall through to unchanged selected-image reuse.
                    admission::ordered(budget, result.scopes.len(), size_of_val(&key))?;
                    result
                        .scopes
                        .insert_cloned_native_for_request(&key, &pair, execution)
                        .map_err(admission::map)?;
                }
            }
            for (key, _) in packet.impacts.iter() {
                admission::ordered(budget, result.scopes.len(), size_of_val(key))?;
                if result.scopes.contains_key(key) {
                    continue;
                }
                // The complete exact roster excludes this program/scope from
                // semantic changes. Authored no-ops may still impact it.
                admission::work(budget, 1)?;
                let program = packet
                    .catalog
                    .program(key.program)
                    .ok_or(Denial::ForeignSource)?;
                let payload = image
                    .ok_or(Denial::ForeignSource)?
                    .history_payload_receipt(&execution, program, key.scope, budget)?;
                let pair = match payload {
                    Some(payload) => HistoricalSourcePair::Complete {
                        before: payload.clone(),
                        after: payload,
                    },
                    None => HistoricalSourcePair::Conservative,
                };
                admission::ordered(budget, result.scopes.len(), size_of_val(key))?;
                result
                    .scopes
                    .insert_cloned_native_for_request(key, &pair, execution)
                    .map_err(admission::map)?;
            }
            Ok(())
        })?;
        Ok(result)
    }
}
