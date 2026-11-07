use super::{execution_tags::*, reuse_tags::*};
use super::{Denial, Read, Write};
use crate::data::node::*;
use crate::data::reuse::{ArtifactEquivalenceContract, NodeReuseContract};

pub(super) fn write(out: &mut Write<'_>, value: &NodeContract) -> Result<(), Denial> {
    let NodeContract {
        semantics,
        projection,
        execution,
        authority,
        reuse,
    } = value;
    let NodeSemanticContract {
        reads,
        produces,
        partition_scope,
        required_context,
    } = semantics;
    out.mask(*reads)?;
    out.mask(*produces)?;
    out.scopes(partition_scope.as_deref())?;
    write_context(out, *required_context)?;
    let NodeProjectionContract {
        consumes,
        consumes_partitions,
    } = projection;
    out.mask(*consumes)?;
    out.scopes(consumes_partitions.as_deref())?;
    let NodeExecutionContract {
        bounded_inputs,
        max_checked_result_heap_bytes,
        equivalence,
        path_class,
        maintenance_mode,
        artifact_policy,
    } = execution;
    out.boolean(bounded_inputs.is_some())?;
    if let Some(inputs) = bounded_inputs {
        out.count(inputs.as_slice().len())?;
        for input in inputs.as_slice() {
            out.node(input.source)?;
            out.byte(input.aspect.index() as u8)?;
            out.scope(input.scope.as_ref())?;
        }
    }
    out.boolean(max_checked_result_heap_bytes.is_some())?;
    if let Some(limit) = max_checked_result_heap_bytes {
        out.number(*limit)?;
    }
    let EquivalenceContract {
        identity_basis,
        suppression_basis,
        canonical_dependency_order,
        comparator_basis,
    } = equivalence;
    write_identity_basis(out, *identity_basis)?;
    write_suppression_basis(out, *suppression_basis)?;
    write_dependency_order(out, *canonical_dependency_order)?;
    write_comparator_basis(out, *comparator_basis)?;
    write_path_class(out, *path_class)?;
    write_maintenance(out, *maintenance_mode)?;
    write_artifact_policy(out, *artifact_policy)?;
    let NodeAuthorityContract { policy } = authority;
    write_authority_policy(out, *policy)?;
    let NodeReuseContract {
        equivalence,
        retain_certification,
    } = reuse;
    let ArtifactEquivalenceContract {
        required_boundaries,
        supported_strategies,
        allows_snapshot_restore_reuse,
        allows_authority_reconciliation_reuse,
    } = equivalence;
    out.count(required_boundaries.len())?;
    for value in required_boundaries {
        write_artifact_boundary(out, *value)?;
    }
    out.count(supported_strategies.len())?;
    for value in supported_strategies {
        write_reuse_strategy(out, *value)?;
    }
    out.boolean(*allows_snapshot_restore_reuse)?;
    out.boolean(*allows_authority_reconciliation_reuse)?;
    out.boolean(*retain_certification)
}

pub(super) fn read(read: &mut Read<'_>) -> Result<NodeContract, Denial> {
    let semantics = NodeSemanticContract {
        reads: read.mask()?,
        produces: read.mask()?,
        partition_scope: read.scopes()?,
        required_context: read_context(read)?,
    };
    let projection = NodeProjectionContract {
        consumes: read.mask()?,
        consumes_partitions: read.scopes()?,
    };
    let bounded_inputs = if read.boolean()? {
        Some(BoundedSignalInputs::new(read.list(|read| {
            Ok(DeclaredSignalInput {
                source: read.node_id()?,
                aspect: read.aspect()?,
                scope: read.scope()?,
            })
        })?))
    } else {
        None
    };
    let max_checked_result_heap_bytes = if read.boolean()? {
        Some(read.number()?)
    } else {
        None
    };
    let execution = NodeExecutionContract {
        bounded_inputs,
        max_checked_result_heap_bytes,
        equivalence: EquivalenceContract {
            identity_basis: read_identity_basis(read)?,
            suppression_basis: read_suppression_basis(read)?,
            canonical_dependency_order: read_dependency_order(read)?,
            comparator_basis: read_comparator_basis(read)?,
        },
        path_class: read_path_class(read)?,
        maintenance_mode: read_maintenance(read)?,
        artifact_policy: read_artifact_policy(read)?,
    };
    let authority = NodeAuthorityContract {
        policy: read_authority_policy(read)?,
    };
    let reuse = NodeReuseContract {
        equivalence: ArtifactEquivalenceContract {
            required_boundaries: read.list(read_artifact_boundary)?,
            supported_strategies: read.list(read_reuse_strategy)?,
            allows_snapshot_restore_reuse: read.boolean()?,
            allows_authority_reconciliation_reuse: read.boolean()?,
        },
        retain_certification: read.boolean()?,
    };
    Ok(NodeContract {
        semantics,
        projection,
        execution,
        authority,
        reuse,
    })
}
