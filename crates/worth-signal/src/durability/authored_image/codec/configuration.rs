use super::{Denial, Read, Write};
use crate::data::node::NodeEvaluationConfig;
use crate::logic::transaction::{
    AspectMergePolicyBinding, AspectMergePolicyName, ConflictIsolationPolicyName,
    ConflictPolicyName, DeletionPolicyName, IdentityMatcherName, MergeStrategyName,
    SourceOnlyPolicyName,
};
use crate::schema::data::{
    SignalSchemaBinding, SignalSchemaId, SignalSchemaName, SignalSchemaVersion,
};

pub(super) fn write(out: &mut Write<'_>, value: &NodeEvaluationConfig) -> Result<(), Denial> {
    let NodeEvaluationConfig {
        schema_binding,
        merge_strategy_name,
        conflict_policy_name,
        identity_matcher_name,
        source_only_policy_name,
        deletion_policy_name,
        conflict_isolation_policy_name,
        aspect_merge_policy_bindings,
        contract,
        condition,
        comparator,
        output_equivalence,
        partitioned_output,
    } = value;
    out.boolean(schema_binding.is_some())?;
    if let Some(binding) = schema_binding {
        out.small_number(binding.schema_id().0)?;
        out.text(binding.semantic_name().as_str())?;
        out.small_number(binding.version().major as u32)?;
        out.small_number(binding.version().minor as u32)?;
        out.text(binding.descriptor_digest())?;
    }
    out.optional_text(merge_strategy_name.as_ref().map(|v| v.as_str()))?;
    out.optional_text(conflict_policy_name.as_ref().map(|v| v.as_str()))?;
    out.optional_text(identity_matcher_name.as_ref().map(|v| v.as_str()))?;
    out.optional_text(source_only_policy_name.as_ref().map(|v| v.as_str()))?;
    out.optional_text(deletion_policy_name.as_ref().map(|v| v.as_str()))?;
    out.optional_text(conflict_isolation_policy_name.as_ref().map(|v| v.as_str()))?;
    out.count(aspect_merge_policy_bindings.len())?;
    for binding in aspect_merge_policy_bindings {
        let AspectMergePolicyBinding {
            aspect,
            policy_name,
        } = binding;
        out.byte(aspect.index() as u8)?;
        out.text(policy_name.as_str())?;
    }
    super::contracts::write(out, contract)?;
    super::policies::write_condition(out, condition)?;
    out.boolean(comparator.is_some())?;
    if let Some(comparator) = comparator {
        super::policies::write_comparator(out, comparator)?;
    }
    super::policies::write_output(out, output_equivalence)?;
    out.boolean(*partitioned_output)
}

pub(super) fn read(read: &mut Read<'_>) -> Result<NodeEvaluationConfig, Denial> {
    let schema_binding = if read.boolean()? {
        Some(SignalSchemaBinding::new(
            SignalSchemaId(read.small_number()?),
            SignalSchemaName::new(read.text()?),
            SignalSchemaVersion::new(
                read.small_number()?
                    .try_into()
                    .map_err(|_| Denial::Malformed)?,
                read.small_number()?
                    .try_into()
                    .map_err(|_| Denial::Malformed)?,
            ),
            read.text()?,
        ))
    } else {
        None
    };
    let merge_strategy_name = read.optional_text()?.map(MergeStrategyName::new);
    let conflict_policy_name = read.optional_text()?.map(ConflictPolicyName::new);
    let identity_matcher_name = read.optional_text()?.map(IdentityMatcherName::new);
    let source_only_policy_name = read.optional_text()?.map(SourceOnlyPolicyName::new);
    let deletion_policy_name = read.optional_text()?.map(DeletionPolicyName::new);
    let conflict_isolation_policy_name =
        read.optional_text()?.map(ConflictIsolationPolicyName::new);
    let aspect_merge_policy_bindings = read.list(|read| {
        Ok(AspectMergePolicyBinding::new(
            read.aspect()?,
            AspectMergePolicyName::new(read.text()?),
        ))
    })?;
    let contract = super::contracts::read(read)?;
    let condition = super::policies::read_condition(read)?;
    let comparator = if read.boolean()? {
        Some(super::policies::read_comparator(read)?)
    } else {
        None
    };
    let output_equivalence = super::policies::read_output(read)?;
    let partitioned_output = read.boolean()?;
    Ok(NodeEvaluationConfig {
        schema_binding,
        merge_strategy_name,
        conflict_policy_name,
        identity_matcher_name,
        source_only_policy_name,
        deletion_policy_name,
        conflict_isolation_policy_name,
        aspect_merge_policy_bindings,
        contract,
        condition,
        comparator,
        output_equivalence,
        partitioned_output,
    })
}
