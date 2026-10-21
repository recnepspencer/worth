//! Closed relation ownership does not authorize custom invariant or join changes.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::OwnedCodecRetirementClosure;
use crate::indexes::data::DerivedIndexKind;
use crate::runtime::{
    RelationalRuntime, RelationalSchemaTransitionAdmissionDenial as Denial,
    RelationalSchemaTransitionAdmissionDenialKind as Kind,
};

pub(in crate::durability::authority::recovery::codec_transition) fn admit_retired_relation_consequences(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    retirement: &OwnedCodecRetirementClosure,
) -> Result<(), Denial> {
    let result = execution.in_scope(|lease| {
        worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
            .run_with_checkpoint(lease, None, |context| {
                for id in &retirement.relations {
                    context.checkpoint(1).map_err(|stop| Denial::request(stop.into()))?;
                    let kind = basis.inner.root.partition_state(id.partition_id)
                        .and_then(|partition| partition.relation_arena.get(id))
                        .and_then(|record| record.kind_id())
                        .ok_or_else(|| deny("retiring relation lost its proven kind"))?;
                    for registration in runtime.schema_contract_runtime.custom_invariant_registries.iter() {
                        let access = &registration.descriptor().operational.access;
                        let work = access.read_relation_kinds.len().saturating_add(access.affected_relation_kinds.len()).saturating_add(1);
                        context.checkpoint(u64::try_from(work).map_err(|_| Denial::request(crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow))?)
                            .map_err(|stop| Denial::request(stop.into()))?;
                        if access.read_relation_kinds.contains(&kind) || access.affected_relation_kinds.contains(&kind) {
                            return Err(deny("codec relation retirement with custom invariant consequences is unsupported"));
                        }
                    }
                    runtime.indexes.with_definitions(|indexes| {
                        for index in indexes.values() {
                            context.checkpoint(1).map_err(|stop| Denial::request(stop.into()))?;
                            if matches!(&index.kind, DerivedIndexKind::RelationJoin(join)
                                if join.left().relation_kind() == kind || join.right().relation_kind() == kind) {
                                return Err(deny("codec relation retirement has an unsupported join index consequence"));
                            }
                        }
                        Ok(())
                    })?;
                }
                Ok(())
            })
    }).map_err(|stop| Denial::request(stop.into()))
        .and_then(|result| result.map_err(|stop| Denial::request(stop.into())))
        .and_then(|result| result.0);
    crate::execution::check_request(execution).map_err(Denial::request)?;
    result
}

fn deny(detail: &str) -> Denial {
    Denial::new(Kind::UnsupportedCodecConsequence, detail)
}
