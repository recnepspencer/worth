//! Adoption changes Query support, never Native values or allocation coordinates.
use super::super::failure;
use super::*;

pub(super) fn assert_unchanged(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Lazy>,
    before: &WorthQueryApplicationCheckpoint,
) {
    let after = failure::checkpoint(host);
    host.with_host_advancement(|phase| {
        let before = host
            .runtime()
            .inspect_application_checkpoint(&phase, before.bytes())
            .unwrap();
        let after = host
            .runtime()
            .inspect_application_checkpoint(&phase, after.bytes())
            .unwrap();
        // The Query frame intentionally retains the adopting run's new facts.
        // Compare the complete Native value image, not that Query metadata.
        assert!(
            after.model.native == before.model.native,
            "all Native model/publication/symbol value bytes remain equal"
        );
        assert_eq!(
            after.live_records, before.live_records,
            "no receipt or other live record was created"
        );
        assert_eq!(
            after.current_publication_records,
            before.current_publication_records
        );
        assert_eq!(
            after.frontiers, before.frontiers,
            "no Native record allocation"
        );
        assert_eq!(
            after.generations, before.generations,
            "no Native identity reuse"
        );
        assert_eq!(
            after.symbol_identity_frontier, before.symbol_identity_frontier,
            "no receipt symbols or other symbol identities allocated"
        );
    })
    .unwrap();
}
