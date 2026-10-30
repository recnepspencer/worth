use std::marker::PhantomData;

use super::WorthQueryDomainOperationSemanticClosure;

type DomainOperationMarker<D, O, F> = fn() -> (D, O, F);

pub struct WorthQueryDomainOperationRef<D, O, F> {
    identity: WorthQueryDomainOperationIdentity,
    canonical_identity: String,
    marker: PhantomData<DomainOperationMarker<D, O, F>>,
}

impl<D, O, F> WorthQueryDomainOperationRef<D, O, F> {
    pub fn identity(&self) -> &WorthQueryDomainOperationIdentity {
        &self.identity
    }

    pub fn canonical_identity(&self) -> &str {
        &self.canonical_identity
    }
}

impl<D, O, F> Clone for WorthQueryDomainOperationRef<D, O, F> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            canonical_identity: self.canonical_identity.clone(),
            marker: PhantomData,
        }
    }
}

impl<D, O, F> std::fmt::Debug for WorthQueryDomainOperationRef<D, O, F> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryDomainOperationRef")
            .field("identity", &self.identity)
            .field("canonical_identity", &self.canonical_identity)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryDomainOperationIdentity {
    name: String,
    version: u32,
}

impl WorthQueryDomainOperationIdentity {
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn slot(&self) -> String {
        format!("{}:{}", self.name, self.version)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableDomainOperationDefinition {
    identity: WorthQueryDomainOperationIdentity,
    semantics: WorthQueryDomainOperationSemanticClosure,
    canonical_identity: String,
}

impl WorthQueryPortableDomainOperationDefinition {
    pub(crate) fn reconstruct_exact(
        identity: WorthQueryDomainOperationIdentity,
        semantics: WorthQueryDomainOperationSemanticClosure,
    ) -> Result<Self, ()> {
        let mut canonical = semantics.clone();
        canonicalize_semantics(&mut canonical);
        if canonical != semantics {
            return Err(());
        }
        let canonical_identity =
            super::canonical_identity::canonical_operation_identity(&identity, &semantics);
        Ok(Self {
            identity,
            semantics,
            canonical_identity,
        })
    }

    pub fn identity(&self) -> &WorthQueryDomainOperationIdentity {
        &self.identity
    }

    pub fn semantics(&self) -> &WorthQueryDomainOperationSemanticClosure {
        &self.semantics
    }

    pub fn canonical_identity(&self) -> &str {
        &self.canonical_identity
    }

    pub(crate) fn canonical_encoded_bytes(&self) -> u64 {
        super::canonical_identity::canonical_operation_encoded_bytes(
            &self.identity,
            &self.semantics,
        )
        .saturating_add(
            self.semantics
                .canonical_query
                .portable_record_logical_bytes(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainOperationDefinition<D, O, F> {
    portable: WorthQueryPortableDomainOperationDefinition,
    marker: PhantomData<DomainOperationMarker<D, O, F>>,
}

impl<D, O, F> WorthQueryDomainOperationDefinition<D, O, F> {
    pub fn new(
        identity: WorthQueryDomainOperationIdentity,
        mut semantics: WorthQueryDomainOperationSemanticClosure,
    ) -> Self {
        canonicalize_semantics(&mut semantics);
        let canonical_identity =
            super::canonical_identity::canonical_operation_identity(&identity, &semantics);
        Self {
            portable: WorthQueryPortableDomainOperationDefinition {
                identity,
                semantics,
                canonical_identity,
            },
            marker: PhantomData,
        }
    }

    pub fn identity(&self) -> &WorthQueryDomainOperationIdentity {
        self.portable.identity()
    }

    pub fn semantics(&self) -> &WorthQueryDomainOperationSemanticClosure {
        self.portable.semantics()
    }

    pub fn reference(&self) -> WorthQueryDomainOperationRef<D, O, F> {
        WorthQueryDomainOperationRef {
            identity: self.portable.identity.clone(),
            canonical_identity: self.portable.canonical_identity.clone(),
            marker: PhantomData,
        }
    }

    pub fn into_portable(self) -> WorthQueryPortableDomainOperationDefinition {
        self.portable
    }
}

fn canonicalize_semantics(semantics: &mut WorthQueryDomainOperationSemanticClosure) {
    semantics.required_capabilities.sort();
    semantics.required_capabilities.dedup();
    semantics.required_domains.sort();
    semantics.required_domains.dedup();
    if let super::WorthQueryOperationGraphReadContract::Declared { roles } =
        &mut semantics.graph_reads
    {
        roles.sort_by(|left, right| left.role().cmp(right.role()));
        roles.dedup();
    }
    if let super::WorthQueryOperationGraphReadContract::DeclaredDomain { roles } =
        &mut semantics.graph_reads
    {
        for role in roles.iter_mut() {
            role.semantic_reads
                .sort_by(super::WorthQueryOperationNativeProjectionContract::canonical_order);
            role.semantic_reads.dedup();
        }
        roles.sort_by(|left, right| left.role.cmp(&right.role));
        roles.dedup();
    }
    if let super::WorthQueryOperationParameterContract::Declared { fields } =
        &mut semantics.parameters
    {
        fields.sort();
        fields.dedup();
    }
    semantics.terminal.result_states.sort();
    semantics.terminal.result_states.dedup();
    semantics.terminal.failure_classes.sort();
    semantics.terminal.failure_classes.dedup();
    canonicalize_touch_contract(&mut semantics.touches);
    canonicalize_effect_contract(&mut semantics.effects);
    canonicalize_invariant_contract(&mut semantics.invariants);
    semantics.workflow.canonicalize();
    super::conditional_node::canonicalize_conditional_nodes(&mut semantics.conditional_nodes);
    derive_conditional_support_requirements(semantics);
}

fn derive_conditional_support_requirements(
    semantics: &mut WorthQueryDomainOperationSemanticClosure,
) {
    let mut has_conditional_node = !semantics.conditional_nodes.is_empty();
    let mut requires_temporal_or_on_demand = semantics.conditional_nodes.iter().any(|node| {
        matches!(
            node.trigger(),
            super::WorthQueryConditionalTrigger::OnDemand(_)
                | super::WorthQueryConditionalTrigger::Temporal(_)
        )
    });
    if let super::WorthQueryOperationWorkflowContract::Declared(workflow) = &semantics.workflow {
        for stage in workflow.stages() {
            has_conditional_node |= !stage.semantics().conditional_nodes.is_empty();
            requires_temporal_or_on_demand |=
                stage.semantics().conditional_nodes.iter().any(|node| {
                    matches!(
                        node.trigger(),
                        super::WorthQueryConditionalTrigger::OnDemand(_)
                            | super::WorthQueryConditionalTrigger::Temporal(_)
                    )
                });
        }
    }
    if has_conditional_node {
        semantics.support.conditional_evaluation = super::WorthQuerySupportRequirement::Required;
        semantics.support.conditional_comparator = super::WorthQuerySupportRequirement::Required;
        semantics.support.conditional_trigger = super::WorthQuerySupportRequirement::Required;
    }
    if requires_temporal_or_on_demand {
        semantics.support.conditional_temporal_or_on_demand =
            super::WorthQuerySupportRequirement::Required;
    }
}

fn canonicalize_touch_contract(contract: &mut super::WorthQueryOperationTouchContract) {
    if let super::WorthQueryOperationTouchContract::Declared {
        graph_roles,
        scopes,
    } = contract
    {
        graph_roles.sort();
        graph_roles.dedup();
        scopes.sort_by(super::WorthQueryOperationTouchScope::canonical_order);
        scopes.dedup();
    }
}

fn canonicalize_effect_contract(contract: &mut super::WorthQueryOperationEffectContract) {
    if let super::WorthQueryOperationEffectContract::Declared { effect_families } = contract {
        effect_families.sort();
        effect_families.dedup();
    }
}

fn canonicalize_invariant_contract(contract: &mut super::WorthQueryOperationInvariantContract) {
    if let super::WorthQueryOperationInvariantContract::Declared { invariant_slots } = contract {
        invariant_slots.sort();
        invariant_slots.dedup();
    }
}
