use std::any::TypeId;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision, ApplicationSemanticDescription,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

mod admission;
mod compatibility;
mod retirement;

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod compatibility_tests;
#[cfg(test)]
mod installation_denial_tests;

pub use admission::{WorthQueryProgramSupportAdmission, WorthQueryProgramSupportDenial};
pub use compatibility::WorthQueryProgramRuleKey;
pub use retirement::{
    WorthQueryProgramSupportPartialRetirementInventory, WorthQueryProgramSupportRetirementDenial,
    WorthQueryProgramSupportRetirementInventory,
};

/// One supported program as the host retains it, erased of its authoring Rust
/// types.
///
/// An entry records what selection needs at runtime: the canonical revision it
/// answers to, the authored program identity, the rule contracts it declares,
/// and the operations and mutation bindings it acts through. It is descriptive
/// support evidence: it establishes neither branch activation nor publication
/// authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramSupportEntry {
    revision: ApplicationProgramRevision,
    identity: ApplicationProgramIdentity,
    rules: Box<[WorthQueryProgramRuleKey]>,
    action_operations: Box<[TypeId]>,
    mutation_bindings: Box<[TypeId]>,
    semantic_description: ApplicationSemanticDescription,
    effectful_action_subjects: Box<[String]>,
}

impl WorthQueryProgramSupportEntry {
    fn admitted(
        revision: ApplicationProgramRevision,
        identity: ApplicationProgramIdentity,
        rules: BTreeSet<WorthQueryProgramRuleKey>,
        action_operations: BTreeSet<TypeId>,
        mutation_bindings: BTreeSet<TypeId>,
        semantic_description: ApplicationSemanticDescription,
        effectful_action_subjects: BTreeSet<String>,
    ) -> Self {
        Self {
            revision,
            identity,
            rules: rules.into_iter().collect(),
            action_operations: action_operations.into_iter().collect(),
            mutation_bindings: mutation_bindings.into_iter().collect(),
            semantic_description,
            effectful_action_subjects: effectful_action_subjects.into_iter().collect(),
        }
    }

    pub fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }

    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }

    pub fn rules(&self) -> &[WorthQueryProgramRuleKey] {
        &self.rules
    }

    pub fn action_operations(&self) -> &[TypeId] {
        &self.action_operations
    }

    pub fn mutation_bindings(&self) -> &[TypeId] {
        &self.mutation_bindings
    }

    pub fn semantic_description(&self) -> &ApplicationSemanticDescription {
        &self.semantic_description
    }

    pub(crate) fn effectful_action_subjects(&self) -> &[String] {
        &self.effectful_action_subjects
    }

    /// Whether this program declares the named installed rule contract, which
    /// is what decides if the rule speaks for a candidate running under it.
    pub fn declares_rule(&self, rule: &WorthQueryProgramRuleKey) -> bool {
        self.rules.contains(rule)
    }

    pub fn acts_through_operation(&self, operation: TypeId) -> bool {
        self.action_operations.contains(&operation)
    }

    pub fn acts_through_mutation_binding(&self, binding: TypeId) -> bool {
        self.mutation_bindings.contains(&binding)
    }

    /// Logical bytes retained so this exact program can still be interpreted.
    /// The count covers owned canonical strings and fixed identity slots; it is
    /// stable across allocator implementations and therefore suitable for
    /// installed resource accounting rather than heap introspection.
    pub fn retained_bytes(&self) -> usize {
        let rule_bytes = self.rules.iter().fold(0usize, |bytes, rule| {
            bytes.saturating_add(rule.identity().len())
        });
        let fact_bytes = self
            .semantic_description
            .facts()
            .iter()
            .fold(0usize, |bytes, fact| {
                bytes
                    .saturating_add(fact.subject().len())
                    .saturating_add(fact.canonical_meaning().len())
            });
        let effect_bytes = self
            .effectful_action_subjects
            .iter()
            .fold(0usize, |bytes, subject| bytes.saturating_add(subject.len()));
        std::mem::size_of::<ApplicationProgramRevision>()
            .saturating_add(self.identity.as_str().len())
            .saturating_add(rule_bytes)
            .saturating_add(
                self.action_operations
                    .len()
                    .saturating_mul(std::mem::size_of::<TypeId>()),
            )
            .saturating_add(
                self.mutation_bindings
                    .len()
                    .saturating_mul(std::mem::size_of::<TypeId>()),
            )
            .saturating_add(fact_bytes)
            .saturating_add(effect_bytes)
    }
}

/// The complete, immutable set of programs one host supports on one installed
/// schema.
///
/// A roster is minted only by [`WorthQueryProgramSupportAdmission::close`], so
/// holding one proves every rostered program fits the installed contracts and
/// that no installed rule is left without a declaring owner. It names no
/// branch and grants no activation.
pub struct WorthQueryProgramSupportRoster<Schema> {
    schema_binding: ApplicationSchemaBindingIdentity,
    entries: Box<[WorthQueryProgramSupportEntry]>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryProgramSupportRoster<Schema> {
    pub(super) fn admitted(
        schema_binding: ApplicationSchemaBindingIdentity,
        entries: Vec<WorthQueryProgramSupportEntry>,
    ) -> Self {
        Self {
            schema_binding,
            entries: entries.into_boxed_slice(),
            marker: PhantomData,
        }
    }

    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn entries(&self) -> &[WorthQueryProgramSupportEntry] {
        &self.entries
    }

    /// Resolves the supported program answering to one canonical revision.
    pub fn entry(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<&WorthQueryProgramSupportEntry> {
        self.entries
            .iter()
            .find(|entry| entry.revision() == revision)
    }
}
