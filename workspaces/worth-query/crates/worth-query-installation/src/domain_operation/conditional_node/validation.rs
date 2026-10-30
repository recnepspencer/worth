use super::*;

pub(crate) fn canonicalize_conditional_nodes(
    nodes: &mut [WorthQueryPortableConditionalNodeDeclaration],
) {
    for node in nodes.iter_mut() {
        node.canonicalize();
    }
    nodes.sort_by(|left, right| left.identity().cmp(right.identity()));
}

pub(crate) fn validate_conditional_nodes(
    nodes: &[WorthQueryPortableConditionalNodeDeclaration],
) -> Result<(), &'static str> {
    for (index, node) in nodes.iter().enumerate() {
        validate_conditional_node(node)?;
        if nodes[..index]
            .iter()
            .any(|prior| prior.identity() == node.identity())
        {
            return Err("duplicate-conditional-node-identity");
        }
    }
    Ok(())
}

pub(crate) fn validate_conditional_node(
    node: &WorthQueryPortableConditionalNodeDeclaration,
) -> Result<(), &'static str> {
    if node.identity().trim().is_empty()
        || node.identity().trim() != node.identity()
        || node.identity().chars().any(char::is_whitespace)
    {
        return Err("invalid-conditional-node-identity");
    }
    if node.outputs().is_empty() {
        return Err("empty-conditional-node-output-set");
    }
    if !node.dependency_comparator().is_portable()
        || !node.output_equivalence().is_portable()
        || !node.artifact_reuse_equivalence().is_portable()
        || node
            .condition()
            .portable_family_identity()
            .is_some_and(|identity| !identity.is_portable())
        || matches!(node.trigger(), WorthQueryConditionalTrigger::OnDemand(identity) if !identity.is_portable())
    {
        return Err("invalid-portable-conditional-family-identity");
    }
    if node
        .condition()
        .temporal_condition()
        .is_some_and(|condition| !condition.duration_is_valid())
    {
        return Err("invalid-temporal-condition-duration");
    }
    match (node.artifact(), node.artifact_reuse_equivalence()) {
        (WorthQueryArtifactPosture::Ephemeral, WorthQueryArtifactReuseEquivalence::NotReusable)
        | (
            WorthQueryArtifactPosture::ReusableWhenEquivalent | WorthQueryArtifactPosture::Durable,
            WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent
            | WorthQueryArtifactReuseEquivalence::OutputEquivalent
            | WorthQueryArtifactReuseEquivalence::Registered(_),
        ) => {}
        _ => return Err("artifact-posture-reuse-equivalence-mismatch"),
    }
    if node.outputs().iter().any(|output| {
        matches!(
            output,
            WorthQueryConditionalNodeOutput::DerivedAspect { consequences, .. }
                if consequences.is_empty()
        )
    }) {
        return Err("derived-conditional-output-lacks-consequence-role");
    }
    if node
        .condition()
        .dependencies()
        .iter()
        .any(|dependency| !node.dependencies().contains(dependency))
    {
        return Err("condition-references-undeclared-dependency");
    }
    validate_delta_threshold_value_family(node)?;
    if node.outputs().iter().any(|output| {
        let WorthQueryConditionalNodeOutput::DerivedAspect { contract, .. } = output else {
            return false;
        };
        node.dependencies()
            .iter()
            .any(|dependency| dependency.contract() == contract)
    }) {
        return Err("conditional-node-output-reenters-trigger-dependency");
    }
    validate_trigger_and_maintenance(node)?;
    validate_output_relationship(node)
}

fn validate_delta_threshold_value_family(
    node: &WorthQueryPortableConditionalNodeDeclaration,
) -> Result<(), &'static str> {
    let Some((dependency, threshold)) = node.condition().delta_threshold_contract() else {
        return Ok(());
    };
    let scalar = match dependency.contract().shape() {
        worth_foundational::facade::AspectShape::Scalar(scalar) => Some(*scalar),
        worth_foundational::facade::AspectShape::Struct(shape)
            if !dependency.projection_mask().is_whole_aspect()
                && dependency.projection_mask().paths().len() == 1
                && dependency.projection_mask().paths()[0].fields().len() == 1 =>
        {
            shape
                .field(&dependency.projection_mask().paths()[0].fields()[0])
                .map(|field| field.value_type())
        }
        _ => None,
    }
    .ok_or("delta-threshold-requires-one-numeric-scalar")?;
    let family = match scalar {
        worth_foundational::facade::ScalarAspectType::Int8
        | worth_foundational::facade::ScalarAspectType::Int16
        | worth_foundational::facade::ScalarAspectType::Int32
        | worth_foundational::facade::ScalarAspectType::Int64
        | worth_foundational::facade::ScalarAspectType::UInt8
        | worth_foundational::facade::ScalarAspectType::UInt16
        | worth_foundational::facade::ScalarAspectType::UInt32
        | worth_foundational::facade::ScalarAspectType::UInt64 => {
            WorthQueryQuantityValueFamily::Integer
        }
        worth_foundational::facade::ScalarAspectType::Float32 => {
            WorthQueryQuantityValueFamily::Float32
        }
        worth_foundational::facade::ScalarAspectType::Float64 => {
            WorthQueryQuantityValueFamily::Float64
        }
        _ => return Err("delta-threshold-requires-one-numeric-scalar"),
    };
    (family == threshold.value_family())
        .then_some(())
        .ok_or("delta-threshold-dependency-value-family-mismatch")
}

fn validate_trigger_and_maintenance(
    node: &WorthQueryPortableConditionalNodeDeclaration,
) -> Result<(), &'static str> {
    use super::condition::ConditionalTriggerClass;
    match (node.condition().trigger_class(), node.trigger()) {
        (ConditionalTriggerClass::OnDemand, WorthQueryConditionalTrigger::OnDemand(_))
        | (ConditionalTriggerClass::Temporal, WorthQueryConditionalTrigger::Temporal(_))
        | (ConditionalTriggerClass::DependencyOrExternal, _) => {}
        (ConditionalTriggerClass::OnDemand, _) => {
            return Err("on-demand-condition-trigger-mismatch")
        }
        (ConditionalTriggerClass::Temporal, _) => {
            return Err("temporal-condition-trigger-mismatch")
        }
    }
    if let (Some(condition), WorthQueryConditionalTrigger::Temporal(wake)) =
        (node.condition().temporal_condition(), node.trigger())
    {
        let expected = match condition {
            WorthQueryTemporalCondition::AtOrAfterUnixNanoseconds(_) => {
                WorthQueryTemporalWake::WallClock
            }
            WorthQueryTemporalCondition::SnapshotAdvance => {
                WorthQueryTemporalWake::OnSnapshotAdvance
            }
            _ => WorthQueryTemporalWake::MonotonicClock,
        };
        if *wake != expected {
            return Err("temporal-condition-wake-mismatch");
        }
    }
    match node.trigger() {
        WorthQueryConditionalTrigger::DependencyChange if node.dependencies().is_empty() => {
            Err("dependency-trigger-requires-dependency")
        }
        WorthQueryConditionalTrigger::OnDemand(_)
            if node.maintenance() != WorthQueryMaintenancePosture::OnDemandOnly =>
        {
            Err("on-demand-trigger-maintenance-mismatch")
        }
        WorthQueryConditionalTrigger::Temporal(_)
            if node.maintenance() != WorthQueryMaintenancePosture::Temporal =>
        {
            Err("temporal-trigger-maintenance-mismatch")
        }
        _ => Ok(()),
    }
}

fn validate_output_relationship(
    node: &WorthQueryPortableConditionalNodeDeclaration,
) -> Result<(), &'static str> {
    match (node.role(), node.output_relationship()) {
        (
            WorthQueryConditionalNodeRole::WorkflowStage,
            WorthQueryOutputRelationship::IsOperationOutput,
        )
        | (
            WorthQueryConditionalNodeRole::Computed,
            WorthQueryOutputRelationship::IsWorkflowStageOutput,
        )
        | (
            WorthQueryConditionalNodeRole::OperationGate,
            WorthQueryOutputRelationship::IsWorkflowStageOutput,
        ) => Err("conditional-node-output-relationship-role-mismatch"),
        _ => {
            let output_present = match node.output_relationship() {
                WorthQueryOutputRelationship::IsOperationOutput => {
                    node.outputs().iter().any(|output| {
                        matches!(
                            output,
                            WorthQueryConditionalNodeOutput::OperationOutput { .. }
                        )
                    })
                }
                WorthQueryOutputRelationship::IsWorkflowStageOutput => {
                    node.outputs().iter().any(|output| {
                        matches!(
                            output,
                            WorthQueryConditionalNodeOutput::WorkflowStageOutput { .. }
                        )
                    })
                }
                WorthQueryOutputRelationship::IntermediateOnly => {
                    node.outputs().iter().all(|output| {
                        matches!(
                            output,
                            WorthQueryConditionalNodeOutput::DerivedAspect { consequences, .. }
                                if consequences.iter().all(|consequence| matches!(
                                    consequence,
                                    WorthQueryConditionalConsequenceRole::DerivedOnly
                                ))
                        )
                    })
                }
                WorthQueryOutputRelationship::ContributesToOperationOutput => {
                    node.outputs().iter().any(|output| {
                        matches!(
                            output,
                            WorthQueryConditionalNodeOutput::OperationOutput { .. }
                        )
                    })
                }
            };
            output_present
                .then_some(())
                .ok_or("conditional-node-output-relationship-missing-output")
        }
    }
}
