//! Exact source address issued after native source disclosure and route selection.
//!
//! This address names a dependency, not a completed output or permission to
//! execute. Its partition survives the first publication and a change of the
//! family's Initial/Preserve binding; changing source epochs are values.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_runtime_world::facade::ProductBranchIncarnation;

use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity;
use crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity;
use crate::ports::SourceReadClosurePort;

/// Only native source qualification constructs this stable family address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputSourceAddress {
    runtime: u64,
    schema: ApplicationSchemaBindingIdentity,
    occurrence: ProductBranchIncarnation,
    scope: WorthQueryOperationScopeEntityBinding,
    family: OutputFamilyIdentity,
    partition: [u8; 32],
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

/// A complete native query occurrence and selected installed family route.
/// It deliberately contains no Current/Pending classification.
pub struct QualifiedOutputSource {
    address: OutputSourceAddress,
    source_epoch: WorthQueryRuntimeSourceIdentity,
    generation: u64,
    lifecycle: super::WorthQueryProducerLifecyclePosture,
    read_closure: Arc<dyn SourceReadClosurePort>,
}

impl QualifiedOutputSource {
    pub fn from_parts(
        address: OutputSourceAddress,
        source_epoch: WorthQueryRuntimeSourceIdentity,
        generation: u64,
        lifecycle: super::WorthQueryProducerLifecyclePosture,
        read_closure: Arc<dyn SourceReadClosurePort>,
    ) -> Self {
        Self {
            address,
            source_epoch,
            generation,
            lifecycle,
            read_closure,
        }
    }

    pub fn address(&self) -> &OutputSourceAddress {
        &self.address
    }

    pub fn source_epoch(&self) -> WorthQueryRuntimeSourceIdentity {
        self.source_epoch
    }

    pub fn read_closure(&self) -> &Arc<dyn SourceReadClosurePort> {
        &self.read_closure
    }

    pub fn lifecycle(&self) -> super::WorthQueryProducerLifecyclePosture {
        self.lifecycle
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

impl OutputSourceAddress {
    pub fn from_parts(
        runtime: u64,
        schema: ApplicationSchemaBindingIdentity,
        occurrence: ProductBranchIncarnation,
        scope: WorthQueryOperationScopeEntityBinding,
        family: OutputFamilyIdentity,
        partition: [u8; 32],
    ) -> Self {
        Self {
            runtime,
            schema,
            occurrence,
            scope,
            family,
            partition,
        }
    }

    pub fn runtime(&self) -> u64 {
        self.runtime
    }
    pub fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub fn occurrence(&self) -> ProductBranchIncarnation {
        self.occurrence
    }
    pub fn scope(&self) -> WorthQueryOperationScopeEntityBinding {
        self.scope
    }
    pub fn family(&self) -> &OutputFamilyIdentity {
        &self.family
    }
    pub fn partition(&self) -> [u8; 32] {
        self.partition
    }
}
