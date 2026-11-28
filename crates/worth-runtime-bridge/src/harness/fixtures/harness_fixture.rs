use crate::facade::{
    BridgeAspectRegistration, BridgeLineageContext, BridgeMappingRegistration, BridgeRuntimePolicy,
};

use super::ScenarioRelationalTruth;

#[derive(Debug, Clone)]
pub struct BridgeHarnessFixture {
    policy: BridgeRuntimePolicy,
    relational_truth: Option<ScenarioRelationalTruth>,
    ambiguous_lineage: bool,
    mappings: Vec<BridgeMappingRegistration>,
    aspect_mappings: Vec<BridgeAspectRegistration>,
    source_declarations: Vec<crate::source::SourceDeclaration>,
    source_adapter_capabilities: Option<crate::source::BridgeSourceCapabilitySet>,
    lineage_context: Option<BridgeLineageContext>,
}

impl BridgeHarnessFixture {
    pub fn new(mappings: Vec<BridgeMappingRegistration>) -> Self {
        Self {
            policy: BridgeRuntimePolicy::development(),
            relational_truth: None,
            ambiguous_lineage: false,
            mappings,
            aspect_mappings: Vec::new(),
            source_declarations: Vec::new(),
            source_adapter_capabilities: None,
            lineage_context: None,
        }
    }

    pub fn with_relational_truth(mut self, truth: ScenarioRelationalTruth) -> Self {
        self.relational_truth = Some(truth);
        self
    }

    pub fn relational_truth(&self) -> Option<&ScenarioRelationalTruth> {
        self.relational_truth.as_ref()
    }

    pub fn first_commit_identity(&self) -> Option<&crate::facade::TruthCommitIdentity> {
        self.relational_truth
            .as_ref()
            .map(|truth| &truth.first_commit)
    }

    pub fn with_ambiguous_lineage(mut self) -> Self {
        self.ambiguous_lineage = true;
        self
    }

    pub fn ambiguous_lineage(&self) -> bool {
        self.ambiguous_lineage
    }

    pub fn with_policy(mut self, policy: BridgeRuntimePolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn with_aspect_mapping(mut self, aspect_mapping: BridgeAspectRegistration) -> Self {
        self.aspect_mappings.push(aspect_mapping);
        self
    }

    pub fn with_source_declaration(
        mut self,
        source_declaration: crate::source::SourceDeclaration,
    ) -> Self {
        self.source_declarations.push(source_declaration);
        self
    }

    pub fn with_source_adapter_capabilities(
        mut self,
        capabilities: crate::source::BridgeSourceCapabilitySet,
    ) -> Self {
        self.source_adapter_capabilities = Some(capabilities);
        self
    }

    pub fn with_lineage_context(mut self, lineage_context: BridgeLineageContext) -> Self {
        self.lineage_context = Some(lineage_context);
        self
    }

    pub fn policy(&self) -> BridgeRuntimePolicy {
        self.policy
    }

    pub fn mappings(&self) -> &[BridgeMappingRegistration] {
        &self.mappings
    }

    pub fn aspect_mappings(&self) -> &[BridgeAspectRegistration] {
        &self.aspect_mappings
    }

    pub fn source_declarations(&self) -> &[crate::source::SourceDeclaration] {
        &self.source_declarations
    }

    pub fn source_adapter_capabilities(&self) -> Option<&crate::source::BridgeSourceCapabilitySet> {
        self.source_adapter_capabilities.as_ref()
    }

    pub fn lineage_context(&self) -> Option<&BridgeLineageContext> {
        self.lineage_context.as_ref()
    }
}
