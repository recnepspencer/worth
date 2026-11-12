//! A settled family's route address is distinct from its currentness evidence.
use super::*;
impl WorthQueryOutputDemandSettlement {
    pub(in crate::domain_computation::primary_graph) fn source_route(
        &self,
    ) -> Option<(
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        [u8; 32],
    )> {
        if let Some(receipt) = self.application_commit_receipt() {
            return Some((
                receipt.principal_scope().scope(),
                receipt.idempotency_binding().source_partition_identity()?,
            ));
        }
        if let Some(stable) = &self.stable {
            return Some((
                stable.exact_settlement().source().scope,
                stable.source_partition_identity()?,
            ));
        }
        self.restored_source
            .as_ref()
            .map(|restored| (restored.source_scope, restored.origin.partition()))
    }

    /// Match only the installed route address. Currentness separately verifies
    /// the actual source epoch, version, output witness, roster and presence.
    pub(in crate::domain_computation::primary_graph) fn covers_historical_route(
        &self,
        expected: &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> bool {
        if self.output_family_identity() != expected.family {
            return false;
        }
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(expected.source_scope);
        if let Some(receipt) = self.application_commit_receipt() {
            return receipt.principal_scope().scope() == scope
                && receipt.idempotency_binding().source_partition_identity()
                    == Some(*expected.source_partition.bytes());
        }
        if let Some(stable) = &self.stable {
            return stable.exact_settlement().source().scope == scope
                && stable.source_partition_identity() == Some(*expected.source_partition.bytes());
        }
        self.restored_source.as_ref().is_some_and(|restored| {
            restored.source_scope == scope
                && restored.origin.partition() == *expected.source_partition.bytes()
        })
    }
}
