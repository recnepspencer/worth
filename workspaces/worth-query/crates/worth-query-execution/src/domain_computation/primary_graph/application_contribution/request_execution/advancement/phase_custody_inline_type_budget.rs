//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryAdvancementPhase<'static>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/request_execution/advancement/phase_custody.rs", "WorthQueryAdvancementPhase", 128 * 1024);
        owner.field::<&'static ActiveAdvancement<'static>>("active");
        owner.field::<AdvancementSource>("source");
        owner.field::<Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>>("owner");
    }
}
