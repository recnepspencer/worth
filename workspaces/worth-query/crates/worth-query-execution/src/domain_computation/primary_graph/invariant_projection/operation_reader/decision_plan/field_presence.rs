//! Initialization metadata belongs to installed writers, not read-only decisions.
use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::{
    WorthQueryOperationTouchContract, WorthQueryOperationTouchScope,
};

pub fn may_write(
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
