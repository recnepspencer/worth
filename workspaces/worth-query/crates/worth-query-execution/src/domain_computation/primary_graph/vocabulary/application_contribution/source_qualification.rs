//! Moved whole from `domain_computation::primary_graph::application_contribution::producer::source_qualification`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::output_family_identity::OutputFamilyIdentity;
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding;
use std::cmp::Ordering;
use std::hash::Hash;
use std::hash::Hasher;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_runtime_world::facade::ProductBranchIncarnation;

/// Only native source qualification constructs this stable family address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct OutputSourceAddress {
    pub(in crate::domain_computation::primary_graph) runtime: u64,
    pub(in crate::domain_computation::primary_graph) schema: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) occurrence: ProductBranchIncarnation,
    pub(in crate::domain_computation::primary_graph) scope: WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) family: OutputFamilyIdentity,
    pub(in crate::domain_computation::primary_graph) partition: [u8; 32],
}

impl Hash for OutputSourceAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.runtime.hash(state);
        self.schema.runtime_ordinal().hash(state);
        self.schema.generation().hash(state);
        self.schema.package_identity().hash(state);
        self.schema.schema_identity().hash(state);
        self.occurrence.hash(state);
        self.scope.hash(state);
        self.family.hash(state);
        self.partition.hash(state);
    }
}

impl Ord for OutputSourceAddress {
    fn cmp(&self, other: &Self) -> Ordering {
        let prefix = |address: &Self| {
            (
                address.runtime,
                address.schema.runtime_ordinal(),
                address.schema.generation(),
                *address.schema.package_identity(),
                *address.schema.schema_identity(),
                address.occurrence,
                address.scope,
            )
        };
        prefix(self)
            .cmp(&prefix(other))
            .then_with(|| self.family.cmp(&other.family))
            .then_with(|| self.partition.cmp(&other.partition))
    }
}

impl PartialOrd for OutputSourceAddress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl OutputSourceAddress {
    pub(in crate::domain_computation::primary_graph) fn runtime(&self) -> u64 {
        self.runtime
    }
    pub(in crate::domain_computation::primary_graph) fn schema(
        &self,
    ) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub(in crate::domain_computation::primary_graph) fn occurrence(
        &self,
    ) -> ProductBranchIncarnation {
        self.occurrence
    }
    pub(in crate::domain_computation::primary_graph) fn scope(
        &self,
    ) -> WorthQueryOperationScopeEntityBinding {
        self.scope
    }
    pub(in crate::domain_computation::primary_graph) fn family(&self) -> &OutputFamilyIdentity {
        &self.family
    }
    pub(in crate::domain_computation::primary_graph) fn partition(&self) -> [u8; 32] {
        self.partition
    }
}
