//! Canonical identity of grant tenure, extent liveness and self-grant posture.
//!
//! A workflow-tenure contract without extent status or a self-grant rule emits
//! exactly the components it emitted before standing grants existed, so every
//! previously installed capability identity is unchanged.
use super::canonical_components::{append_field, append_relation, append_value_binding, text};
use super::{
    ApplicationCapabilityCanonicalComponent, ApplicationCapabilityCurrentnessDefinition,
    ApplicationCapabilityDelegationDefinition, ApplicationCapabilityGrantTenure,
    ApplicationCapabilitySelfGrantRule,
};

const STANDING_TENURE: &str = "standing-v1";

pub(super) fn append_tenure(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    currentness: &ApplicationCapabilityCurrentnessDefinition,
) {
    match currentness.tenure() {
        ApplicationCapabilityGrantTenure::Workflow(workflow) => {
            append_field(
                components,
                "constraints.currentness.workflow.grant",
                workflow.grant(),
            );
            append_field(
                components,
                "constraints.currentness.workflow.resource",
                workflow.resource(),
            );
        }
        ApplicationCapabilityGrantTenure::Standing => {
            text(
                components,
                "constraints.currentness.tenure",
                STANDING_TENURE,
            );
        }
    }
}

pub(super) fn append_extent_status(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    currentness: &ApplicationCapabilityCurrentnessDefinition,
) {
    for (ordinal, status) in currentness.extent_status().iter().enumerate() {
        append_value_binding(
            components,
            &format!("constraints.currentness.extent-status.{ordinal}"),
            status,
        );
    }
}

pub(super) fn append_holder(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    delegation: &ApplicationCapabilityDelegationDefinition,
) {
    match delegation.lineage() {
        Some(lineage) => {
            append_relation(components, "delegation.parent", lineage.parent());
            append_relation(components, "delegation.grantor", lineage.grantor());
            append_relation(components, "delegation.grantee", delegation.grantee());
            append_field(components, "delegation.limit", lineage.limit());
        }
        None => {
            text(components, "delegation.lineage", STANDING_TENURE);
            append_relation(components, "delegation.grantee", delegation.grantee());
        }
    }
}

pub(super) fn append_self_grant(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    delegation: &ApplicationCapabilityDelegationDefinition,
) {
    match delegation.self_grant() {
        ApplicationCapabilitySelfGrantRule::Permitted => {}
        ApplicationCapabilitySelfGrantRule::Forbidden => {
            text(components, "delegation.self-grant", "forbidden");
        }
    }
}
