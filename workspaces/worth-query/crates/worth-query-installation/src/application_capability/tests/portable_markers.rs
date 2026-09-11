use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityContextEntitySlotMarkerIdentity,
        ApplicationCapabilityContextMarkerIdentity, ApplicationCapabilityMarkerIdentity,
        ApplicationCapabilityProvenanceMarkerIdentity,
    },
    application_schema::ApplicationOperationMarkerIdentity,
};

pub(super) struct Schema;
pub(super) struct Capability;
pub(super) struct Operation;
pub(super) struct Grant;
pub(super) struct Resource;
pub(super) struct Principal;
pub(super) struct Facts;
pub(super) struct ResourceFacts;
pub(super) struct Action;
pub(super) struct Purpose;
pub(super) struct Field;
pub(super) struct Amount;
pub(super) struct Workflow;
pub(super) struct ResourceWorkflow;
pub(super) struct Status;
pub(super) struct ValidFrom;
pub(super) struct ValidThrough;
pub(super) struct DelegationLimit;
pub(super) struct ResourceRelation;
pub(super) struct ScopedRelation;
pub(super) struct PrincipalResource;
pub(super) struct Parent;
pub(super) struct Grantor;
pub(super) struct Grantee;
pub(super) struct Context;
pub(super) struct OtherContext;
pub(super) struct ChangedContext;
pub(super) struct Provenance;
pub(super) struct OtherProvenance;
pub(super) struct ChangedProvenance;
pub(super) struct ResourceSlot;
pub(super) struct ChangedWorkflow;
pub(super) struct ChangedResourceWorkflow;
pub(super) struct ChangedValidFrom;

worth_query_declaration::worth_query_portable_type!(
    Capability => "worth.query.installation-test.capability.v1"
);

impl ApplicationCapabilityMarkerIdentity for Capability {
    type Schema = Schema;
    const IDENTIFIER: &'static str = "Capability";
}

macro_rules! portable_context_marker {
    ($marker:ty, $identifier:literal, $identity:literal) => {
        worth_query_declaration::worth_query_portable_type!($marker => $identity);
        impl ApplicationCapabilityContextMarkerIdentity for $marker {
            type Schema = Schema;
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

macro_rules! portable_provenance_marker {
    ($marker:ty, $identifier:literal, $identity:literal) => {
        worth_query_declaration::worth_query_portable_type!($marker => $identity);
        impl ApplicationCapabilityProvenanceMarkerIdentity for $marker {
            type Schema = Schema;
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

portable_context_marker!(
    Context,
    "Context",
    "worth.query.installation-test.context.v1"
);
portable_context_marker!(
    OtherContext,
    "Context",
    "worth.query.installation-test.other-context.v1"
);
portable_context_marker!(
    ChangedContext,
    "ChangedContext",
    "worth.query.installation-test.changed-context.v1"
);
portable_provenance_marker!(
    Provenance,
    "Provenance",
    "worth.query.installation-test.provenance.v1"
);
portable_provenance_marker!(
    OtherProvenance,
    "Provenance",
    "worth.query.installation-test.other-provenance.v1"
);
portable_provenance_marker!(
    ChangedProvenance,
    "ChangedProvenance",
    "worth.query.installation-test.changed-provenance.v1"
);
worth_query_declaration::worth_query_portable_type!(ResourceSlot =>
    "worth.query.installation-test.resource-slot.v1");
worth_query_declaration::worth_query_structured_value_binding!(
    pub(super) OperationInputBinding for () { identity: "worth.rust.unit" }
);

impl ApplicationCapabilityContextEntitySlotMarkerIdentity for ResourceSlot {
    type Schema = Schema;
    type Context = Context;
    type Entity = Resource;
    const IDENTIFIER: &'static str = "ResourceSlot";
}

impl ApplicationOperationMarkerIdentity<Schema> for Operation {
    type InputBinding = OperationInputBinding;
    const IDENTIFIER: &'static str = "Operation";
}
