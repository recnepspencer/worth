//! Initialization metadata belongs to installed writers, not read-only decisions.
use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::{
    WorthQueryOperationTouchContract, WorthQueryOperationTouchScope,
};

pub(super) fn may_write(
    contract: &WorthQueryOperationTouchContract,
    entity: &str,
    locator: &AspectFieldLocator,
) -> bool {
    contract.scopes().iter().any(|scope| {
        matches!(scope,
        WorthQueryOperationTouchScope::WriteField(write)
        if write.entity() == entity && write.contract().key() == locator.aspect().aspect_key()
            && write.field_path() == locator.field_path())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, AccountNote, AccountScore, PatchAccountDraftOperation,
        WrongFieldRetentionOperation,
    };

    #[test]
    fn optional_aspect_initialization_presence_follows_exact_installed_write_membership() {
        let world = installed_authorization_world(true);
        let schema = world.application.installed_schema();
        let writer = schema
            .installed_operation(PatchAccountDraftOperation::reference())
            .unwrap();
        let nonwriter = schema
            .installed_operation(WrongFieldRetentionOperation::reference())
            .unwrap();
        let graph = world.application.runtime.primary_graph().unwrap();
        for field in [
            AccountNote::reference().field(),
            AccountScore::reference().field(),
        ] {
            let locator = graph
                .layout
                .field_locator("Account", "AccountPolicy", field)
                .unwrap();
            assert!(may_write(writer.contracts().touches(), "Account", locator));
            assert!(!may_write(
                nonwriter.contracts().touches(),
                "Account",
                locator
            ));
        }
    }
}
