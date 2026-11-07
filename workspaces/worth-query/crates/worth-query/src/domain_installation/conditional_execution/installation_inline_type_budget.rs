//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryConditionalDependencyInstallation>("workspaces/worth-query/crates/worth-query/src/domain_installation/conditional_execution/installation.rs", "WorthQueryConditionalDependencyInstallation", 128 * 1024);
        owner.field::<Option<RelationalBridgeRecordIdentityParts>>("source_record_identity");
        owner.field::<Option<RelationalBridgeRecordIdentityParts>>("observation_record_identity");
        owner.field::<Vec<BridgeSignalAspectTargetDeclaration>>("targets");
    }
}
