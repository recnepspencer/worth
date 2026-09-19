use std::collections::BTreeSet;
use std::sync::Arc;

use super::{
    WorthQueryProvisionalDenialKind, WorthQueryProvisionalFailure,
    WorthQueryProvisionalProposalBasis,
};
use crate::domain_computation::provider_session::WorthQueryFreshDecisionReadSet;
use crate::domain_computation::provider_session::WorthQuerySessionEffectAuthority;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProvisionalEffectAction {
    Create { symbolic_identity: Arc<str> },
    Replace { target_identity: Arc<str> },
    Retire { target_identity: Arc<str> },
    DeriveView { view_identity: Arc<str> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProvisionalEffectStep {
    effect_family: Arc<str>,
    action: WorthQueryProvisionalEffectAction,
    symbolic_dependencies: Vec<Arc<str>>,
    artifact_dependencies: Vec<Arc<str>>,
    proposal_basis: Option<WorthQueryProvisionalProposalBasis>,
}

impl WorthQueryProvisionalEffectStep {
    pub fn new(
        effect_family: impl Into<Arc<str>>,
        action: WorthQueryProvisionalEffectAction,
    ) -> Result<Self, WorthQueryProvisionalFailure> {
        let effect_family = canonical(effect_family)?;
        validate_action(&action)?;
        Ok(Self {
            effect_family,
            action,
            symbolic_dependencies: Vec::new(),
            artifact_dependencies: Vec::new(),
            proposal_basis: None,
        })
    }

    pub fn with_symbolic_dependencies(
        mut self,
        identities: impl IntoIterator<Item = impl Into<Arc<str>>>,
    ) -> Result<Self, WorthQueryProvisionalFailure> {
        self.symbolic_dependencies = canonical_set(identities)?;
        Ok(self)
    }

    pub fn with_artifact_dependencies(
        mut self,
        identities: impl IntoIterator<Item = impl Into<Arc<str>>>,
    ) -> Result<Self, WorthQueryProvisionalFailure> {
        self.artifact_dependencies = canonical_set(identities)?;
        Ok(self)
    }

    pub fn with_proposal_basis(mut self, basis: WorthQueryProvisionalProposalBasis) -> Self {
        self.proposal_basis = Some(basis);
        self
    }

    pub fn effect_family(&self) -> &str {
        &self.effect_family
    }

    pub fn action(&self) -> &WorthQueryProvisionalEffectAction {
        &self.action
    }

    pub fn symbolic_dependencies(&self) -> &[Arc<str>] {
        &self.symbolic_dependencies
    }

    pub fn artifact_dependencies(&self) -> &[Arc<str>] {
        &self.artifact_dependencies
    }

    pub fn proposal_basis(&self) -> Option<&WorthQueryProvisionalProposalBasis> {
        self.proposal_basis.as_ref()
    }
}

pub struct WorthQueryLoweredProvisionalEffectProgram {
    identity: Arc<str>,
    binding_identity: Arc<str>,
    decision_read_set_identity: Arc<str>,
    steps: Arc<[WorthQueryProvisionalEffectStep]>,
}

impl WorthQueryLoweredProvisionalEffectProgram {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn steps(&self) -> &[WorthQueryProvisionalEffectStep] {
        &self.steps
    }

    pub(crate) fn belongs_to(&self, binding_identity: &str) -> bool {
        self.binding_identity.as_ref() == binding_identity
    }

    pub(crate) fn uses_read_set(&self, identity: &str) -> bool {
        self.decision_read_set_identity.as_ref() == identity
    }
}

impl WorthQuerySessionEffectAuthority<'_> {
    pub fn lower_provisional_program(
        &self,
        read_set: &WorthQueryFreshDecisionReadSet,
        steps: impl IntoIterator<Item = WorthQueryProvisionalEffectStep>,
    ) -> Result<WorthQueryLoweredProvisionalEffectProgram, WorthQueryProvisionalFailure> {
        if !read_set.belongs_to(self.binding().canonical_identity()) {
            return Err(WorthQueryProvisionalFailure::new(
                WorthQueryProvisionalDenialKind::SessionBindingMismatch,
                "decision read-set belongs to another provider session",
            ));
        }
        let steps = steps.into_iter().collect::<Vec<_>>();
        // Empty programs are lawful for emit-only / outbox-only commits (R8.55):
        // the application may declare no domain mutation while still co-committing
        // Query scaffolding (idempotency + dispatch outbox) registered on the
        // provider attempt separately from provisional effect steps.
        validate_closure_and_symbols(self, read_set, &steps)?;
        Ok(WorthQueryLoweredProvisionalEffectProgram {
            identity: self.binding().canonical_identity().into(),
            binding_identity: self.binding().canonical_identity().into(),
            decision_read_set_identity: read_set.read_set_identity().into(),
            steps: steps.into(),
        })
    }
}

fn validate_closure_and_symbols(
    authority: &WorthQuerySessionEffectAuthority<'_>,
    read_set: &WorthQueryFreshDecisionReadSet,
    steps: &[WorthQueryProvisionalEffectStep],
) -> Result<(), WorthQueryProvisionalFailure> {
    let mut symbols = BTreeSet::new();
    let mut proposed_fact_identities = BTreeSet::new();
    for step in steps {
        validate_effect_family(authority, step)?;
        validate_symbolic_dependencies(&symbols, step)?;
        validate_artifact_dependencies(authority, step)?;
        register_created_symbol(&mut symbols, step)?;
        register_proposed_fact_identity(&mut proposed_fact_identities, step)?;
        validate_concrete_target(read_set, step)?;
        validate_proposal_basis(authority, step)?;
    }
    Ok(())
}

fn validate_effect_family(
    authority: &WorthQuerySessionEffectAuthority<'_>,
    step: &WorthQueryProvisionalEffectStep,
) -> Result<(), WorthQueryProvisionalFailure> {
    if authority
        .plan()
        .effect_closure()
        .iter()
        .any(|family| family == step.effect_family())
    {
        Ok(())
    } else {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::UndeclaredEffectFamily,
            "effect family is outside the sealed provider plan",
        ))
    }
}

fn validate_symbolic_dependencies(
    symbols: &BTreeSet<Arc<str>>,
    step: &WorthQueryProvisionalEffectStep,
) -> Result<(), WorthQueryProvisionalFailure> {
    if step
        .symbolic_dependencies()
        .iter()
        .any(|symbol| !symbols.contains(symbol))
    {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::UnknownSymbolicReference,
            "symbolic dependency must refer to an earlier creation",
        ))
    } else {
        Ok(())
    }
}

fn validate_artifact_dependencies(
    authority: &WorthQuerySessionEffectAuthority<'_>,
    step: &WorthQueryProvisionalEffectStep,
) -> Result<(), WorthQueryProvisionalFailure> {
    if step.artifact_dependencies().iter().any(|artifact| {
        !authority
            .plan()
            .artifact_closure()
            .iter()
            .any(|declared| declared == artifact.as_ref())
    }) {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::UndeclaredArtifactDependency,
            "artifact dependency is outside the sealed provider plan",
        ))
    } else {
        Ok(())
    }
}

fn register_created_symbol(
    symbols: &mut BTreeSet<Arc<str>>,
    step: &WorthQueryProvisionalEffectStep,
) -> Result<(), WorthQueryProvisionalFailure> {
    let WorthQueryProvisionalEffectAction::Create { symbolic_identity } = step.action() else {
        return Ok(());
    };
    if symbols.insert(Arc::clone(symbolic_identity)) {
        Ok(())
    } else {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::SymbolAlreadyDefined,
            "symbolic identity may be created only once",
        ))
    }
}

fn register_proposed_fact_identity<'a>(
    identities: &mut BTreeSet<&'a str>,
    step: &'a WorthQueryProvisionalEffectStep,
) -> Result<(), WorthQueryProvisionalFailure> {
    if identities.insert(action_identity(step.action())) {
        Ok(())
    } else {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProposedFactIdentityAlreadyDefined,
            "one provisional program cannot define the same proposed fact twice",
        ))
    }
}

fn validate_concrete_target(
    read_set: &WorthQueryFreshDecisionReadSet,
    step: &WorthQueryProvisionalEffectStep,
) -> Result<(), WorthQueryProvisionalFailure> {
    let target = match step.action() {
        WorthQueryProvisionalEffectAction::Replace { target_identity }
        | WorthQueryProvisionalEffectAction::Retire { target_identity } => target_identity,
        WorthQueryProvisionalEffectAction::Create { .. }
        | WorthQueryProvisionalEffectAction::DeriveView { .. } => return Ok(()),
    };
    if read_set.contains_locator(target) {
        Ok(())
    } else {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::UndeclaredTarget,
            "concrete effect target was not observed by the admitted read-set",
        ))
    }
}

fn validate_proposal_basis(
    authority: &WorthQuerySessionEffectAuthority<'_>,
    step: &WorthQueryProvisionalEffectStep,
) -> Result<(), WorthQueryProvisionalFailure> {
    if step
        .proposal_basis()
        .is_some_and(|proposal| !proposal.belongs_to(authority.terminal_binding()))
    {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProposalBasisMismatch,
            "proposal basis does not match the sealed semantic basis",
        ))
    } else {
        Ok(())
    }
}

fn action_identity(action: &WorthQueryProvisionalEffectAction) -> &str {
    match action {
        WorthQueryProvisionalEffectAction::Create { symbolic_identity } => symbolic_identity,
        WorthQueryProvisionalEffectAction::Replace { target_identity } => target_identity,
        WorthQueryProvisionalEffectAction::Retire { target_identity } => target_identity,
        WorthQueryProvisionalEffectAction::DeriveView { view_identity } => view_identity,
    }
}

fn canonical(value: impl Into<Arc<str>>) -> Result<Arc<str>, WorthQueryProvisionalFailure> {
    let value = value.into();
    if value.trim().is_empty() || value.trim() != value.as_ref() {
        return Err(WorthQueryProvisionalFailure::invalid_program(
            "program identities must be non-empty canonical text",
        ));
    }
    Ok(value)
}

fn canonical_set(
    values: impl IntoIterator<Item = impl Into<Arc<str>>>,
) -> Result<Vec<Arc<str>>, WorthQueryProvisionalFailure> {
    let mut values = values
        .into_iter()
        .map(canonical)
        .collect::<Result<Vec<_>, _>>()?;
    values.sort();
    values.dedup();
    Ok(values)
}

fn validate_action(
    action: &WorthQueryProvisionalEffectAction,
) -> Result<(), WorthQueryProvisionalFailure> {
    let identity = match action {
        WorthQueryProvisionalEffectAction::Create { symbolic_identity } => symbolic_identity,
        WorthQueryProvisionalEffectAction::Replace { target_identity } => target_identity,
        WorthQueryProvisionalEffectAction::Retire { target_identity } => target_identity,
        WorthQueryProvisionalEffectAction::DeriveView { view_identity } => view_identity,
    };
    canonical(Arc::clone(identity)).map(|_| ())
}
