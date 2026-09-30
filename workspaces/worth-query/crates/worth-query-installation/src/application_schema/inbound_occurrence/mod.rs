mod contract;
mod validation;

pub use contract::InstalledInboundOccurrenceContract;
pub(crate) use validation::{install_inbound_occurrence, InboundOccurrenceInstallationDenial};

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
    use worth_query_declaration::facade::application_schema::{
        ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
        ApplicationSchemaMember,
    };

    use super::*;

    #[test]
    fn exact_effect_installs_and_old_operation_has_no_inbound_support() {
        let one = NonZeroU64::new(1).unwrap();
        let member = ApplicationSchemaMember::OperationInboundOccurrence {
            operation: "notify".to_owned(),
            effect: "death-notice".to_owned(),
            protocol: ApplicationInboundOccurrenceProtocol::new(
                BoundaryProtocolIdentity::new("bank.rail-completion"),
                BoundaryProtocolVersion::new(1),
            ),
            source_identity: "bank-rail".to_owned(),
            limits: ApplicationInboundOccurrenceLimits {
                maximum_envelope_bytes: one,
                maximum_verifier_work: one,
                maximum_payload_bytes: one,
                maximum_outstanding_dispatch_provenance: one,
                maximum_accepted_occurrences: one,
                maximum_accepted_bytes: one,
                maximum_concurrent_publications: one,
                maximum_discovery_work: one,
                replay_window_milliseconds: one,
                maximum_cleanup_work: one,
            },
        };
        assert!(install_inbound_occurrence(&[], "old", "death-notice")
            .unwrap()
            .is_none());
        let installed =
            install_inbound_occurrence(std::slice::from_ref(&member), "notify", "death-notice")
                .unwrap()
                .unwrap();
        assert_eq!(installed.effect(), "death-notice");
        assert_eq!(installed.source_identity(), "bank-rail");
        assert_eq!(
            install_inbound_occurrence(&[member], "notify", "other-effect"),
            Err(InboundOccurrenceInstallationDenial::Orphaned),
        );
    }
}
