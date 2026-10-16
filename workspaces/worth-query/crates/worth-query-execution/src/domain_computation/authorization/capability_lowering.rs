use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityFieldBinding, ApplicationCapabilityGraphRule,
    ApplicationCapabilityScopeGuard, ErasedApplicationCapabilityContract,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationAuthorizationTraversalDirection,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationFieldComparison, RelationalAuthorizationFieldConstraint,
    RelationalAuthorizationPathPlan, RelationalAuthorizationTraversalDirection,
};
use worth_runtime_bridge::facade::BridgeAuthorizationRuleEffect;

use super::capability_binding_lowering::{
    capability_principal, clause_identity, kind, lower_context_anchors, operand, predicate,
    relation, request_bindings,
};
use super::capability_registry::WorthQueryCapabilityDecisionRuleBindings;
use super::capability_registry::{
    field_binding, WorthQueryCapabilityContextAnchor, WorthQueryCapabilityPathTemplate,
    WorthQueryCapabilityRequestGuard, WorthQueryCapabilityRequestValueAxis,
};
use super::lowering::lower_authorization_path;
use super::{authorization_denial, WorthQueryOperationAuthorizationDenial};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

mod accumulator;
mod elevation;
mod installed_plan;
#[cfg(test)]
mod semantic_model_tests;

use accumulator::WorthQueryCapabilityRuleLoweringAccumulator;
pub(super) use accumulator::WorthQueryCapabilityRuleSet;
pub(super) use installed_plan::{compile_capability_plan, WorthQueryInstalledCapabilityPlan};

const RECORDED_ACTOR_CLAUSE_NAMESPACE: u8 = 0xc8;

fn compile_grant_path(
    capability: &[u8; 32],
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    principal_kind: worth_relational::facade::identity::KindId,
    grant_kind: worth_relational::facade::identity::KindId,
    scope_kind: worth_relational::facade::identity::KindId,
) -> Result<WorthQueryCapabilityPathTemplate, WorthQueryOperationAuthorizationDenial> {
    let grantee = relation(
        layout,
        contract.delegation().grantee(),
        RelationalAuthorizationTraversalDirection::Forward,
    )?;
    if grantee.from_kind() != principal_kind || grantee.to_kind() != grant_kind {
        return Err(authorization_denial(
            contract.name(),
            "capability grantee relation endpoints changed",
        ));
    }
    let resource = relation(
        layout,
        contract.target().resource(),
        RelationalAuthorizationTraversalDirection::Forward,
    )?;
    if resource.from_kind() != grant_kind || resource.to_kind() != scope_kind {
        return Err(authorization_denial(
            contract.name(),
            "capability resource relation endpoints changed",
        ));
    }
    let currentness = contract.constraints().currentness();
    let predicates = vec![
        predicate(layout, 1, grant_kind, contract.target().action())?,
        predicate(layout, 1, grant_kind, contract.target().purpose())?,
        predicate(layout, 1, grant_kind, currentness.active_status())?,
    ];
    let workflow = currentness.workflow();
    let workflow_constraint = RelationalAuthorizationFieldConstraint::new(
        operand(layout, 1, grant_kind, workflow.grant())?,
        RelationalAuthorizationFieldComparison::Equal,
        operand(layout, 2, scope_kind, workflow.resource())?,
    );
    Ok(WorthQueryCapabilityPathTemplate {
        plan: RelationalAuthorizationPathPlan::new([grantee, resource], predicates)
            .with_field_constraints([workflow_constraint]),
        identity: clause_identity(capability, 0),
        guard: WorthQueryCapabilityRequestGuard::Unconditional,
        grant_ordinal: Some(1),
        elevation_ordinals: Vec::new(),
        elevation_resource_ordinal: None,
        context_anchors: Vec::new(),
    })
}

fn compile_composition_rules(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    lowering: &mut WorthQueryCapabilityRuleLoweringAccumulator,
) -> Result<WorthQueryCapabilityDecisionRuleBindings, WorthQueryOperationAuthorizationDenial> {
    let composition = contract.composition();
    let allow = compile_graph_rule(
        contract,
        layout,
        capability,
        BridgeAuthorizationRuleEffect::Required,
        composition.decision().allow().graph(),
        lowering,
    )?;
    let mut compile_optional = |graph: Option<&ApplicationCapabilityGraphRule>| {
        graph
            .map(|graph| {
                compile_graph_rule(
                    contract,
                    layout,
                    capability,
                    BridgeAuthorizationRuleEffect::Prohibited,
                    graph,
                    lowering,
                )
            })
            .transpose()
    };
    let deny = compile_optional(composition.decision().deny().graph())?;
    let conflict = compile_optional(composition.decision().conflict().graph())?;
    let separation_of_duty = compile_optional(composition.actors().separation_of_duty().graph())?;
    let distinct_actor = compile_optional(composition.actors().distinct_actor().graph())?;
    let recorded_actor = composition
        .actors()
        .recorded_actor()
        .requirement()
        .map(|requirement| {
            if !requirement.is_valid() {
                return Err(authorization_denial(
                    contract.name(),
                    "invalid native recorded-role requirement",
                ));
            }
            // Installed capability meaning already binds this exact requirement.
            // Preserve all its identity bits in a namespace distinct from graph
            // paths (0xc7), without deriving a second semantic digest.
            let mut identity = *capability;
            identity[0] ^= RECORDED_ACTOR_CLAUSE_NAMESPACE;
            Ok(lowering.add_recorded_actor(requirement, identity))
        })
        .transpose()?;
    Ok(WorthQueryCapabilityDecisionRuleBindings {
        grant: 0,
        allow,
        deny,
        conflict,
        separation_of_duty,
        distinct_actor,
        recorded_actor,
    })
}

fn compile_graph_rule(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    effect: BridgeAuthorizationRuleEffect,
    graph: &ApplicationCapabilityGraphRule,
    lowering: &mut WorthQueryCapabilityRuleLoweringAccumulator,
) -> Result<usize, WorthQueryOperationAuthorizationDenial> {
    let mut requirements = Vec::with_capacity(graph.requirements().len());
    let mut next_path_index = lowering.path_count();
    for requirement in graph.requirements() {
        let mut paths = Vec::with_capacity(requirement.clauses().len());
        for clause in requirement.clauses() {
            let path_index = next_path_index;
            next_path_index += 1;
            let plan = lower_authorization_path(layout, clause.path())?;
            let guard = lower_guard(contract, clause.guard())?;
            let anchors = lower_context_anchors(contract, layout, clause)?;
            let grant_ordinal = command_grant_ordinal(contract, clause.path(), &anchors)?;
            paths.push(WorthQueryCapabilityPathTemplate {
                plan,
                identity: clause_identity(capability, path_index),
                guard,
                grant_ordinal,
                elevation_ordinals: Vec::new(),
                elevation_resource_ordinal: None,
                context_anchors: anchors,
            });
        }
        requirements.push(paths);
    }
    Ok(lowering.add_rule(effect, requirements).rule_index())
}

fn grant_ordinal(
    contract: &ErasedApplicationCapabilityContract,
    path: &ApplicationAuthorizationPath,
) -> Result<Option<usize>, WorthQueryOperationAuthorizationDenial> {
    let grant = contract.grant_entity();
    let mut found = (path.principal_entity() == grant).then_some(0);
    for (index, traversal) in path.traversals().iter().enumerate() {
        let entity = match traversal.direction() {
            ApplicationAuthorizationTraversalDirection::Forward => traversal.to(),
            ApplicationAuthorizationTraversalDirection::Reverse => traversal.from(),
        };
        if entity == grant && found.replace(index + 1).is_some() {
            return Err(authorization_denial(
                contract.name(),
                "capability policy path traverses the grant kind more than once",
            ));
        }
    }
    Ok(found)
}

fn command_grant_ordinal(
    contract: &ErasedApplicationCapabilityContract,
    path: &ApplicationAuthorizationPath,
    context: &[WorthQueryCapabilityContextAnchor],
) -> Result<Option<usize>, WorthQueryOperationAuthorizationDenial> {
    let grant = grant_ordinal(contract, path)?;
    Ok(grant.filter(|grant| !context.iter().any(|anchor| anchor.ordinal < *grant)))
}

fn lower_guard(
    contract: &ErasedApplicationCapabilityContract,
    guard: &ApplicationCapabilityScopeGuard,
) -> Result<WorthQueryCapabilityRequestGuard, WorthQueryOperationAuthorizationDenial> {
    let [] = guard.requirements() else {
        if guard.requirements().len() != 1 {
            return Err(authorization_denial(
                contract.name(),
                "capability clause guard must bind one request axis",
            ));
        }
        let requirement = &guard.requirements()[0];
        return Ok(WorthQueryCapabilityRequestGuard::Accepted {
            axis: guard_axis(contract, requirement.field())?,
            values: requirement.values().to_vec(),
        });
    };
    Ok(WorthQueryCapabilityRequestGuard::Unconditional)
}

fn guard_axis(
    contract: &ErasedApplicationCapabilityContract,
    field: &ApplicationCapabilityFieldBinding,
) -> Result<WorthQueryCapabilityRequestValueAxis, WorthQueryOperationAuthorizationDenial> {
    if field == contract.target().action().field() {
        return Ok(WorthQueryCapabilityRequestValueAxis::Action);
    }
    if field == contract.target().purpose().field() {
        return Ok(WorthQueryCapabilityRequestValueAxis::Purpose);
    }
    if field_binding(contract.target().field()) == Some(field) {
        return Ok(WorthQueryCapabilityRequestValueAxis::Field);
    }
    if field_binding(contract.constraints().magnitude()) == Some(field) {
        return Ok(WorthQueryCapabilityRequestValueAxis::Magnitude);
    }
    Err(authorization_denial(
        field.field(),
        "capability guard is not bound to a request axis",
    ))
}
