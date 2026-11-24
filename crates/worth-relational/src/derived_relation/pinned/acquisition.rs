//! One controlled acquisition owner serves both old and receipt-bearing doors.
use super::*;

impl PinnedDerivedRelationRead {
    pub(crate) fn acquire_projected(
        projection: &VisibilityProjectionView,
        execution: &ExecutionRequest<'_, '_>,
        definition: &Handle,
        root: NativeMemberKey,
        shape: super::super::DerivedRelationReadShape,
        maximum_work: Option<usize>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        read_projected_source(
            projection,
            execution,
            PinnedDerivedRelationScope::new(definition, root, shape).with_work_limit(maximum_work),
        )
        .map_err(|stop| stop.kind)
    }
}

pub(crate) fn read_projected_source(
    projection: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    scope: PinnedDerivedRelationScope<'_>,
) -> Result<PinnedDerivedRelationRead, PinnedDerivedRelationDenial> {
    let mut reads = Reads::default();
    let mut work = 0;
    let mut inner_denial = None;
    let result = execution.run_native_with_checkpoint(|context| {
        let result = (|| {
            let mut read = PinnedSource {
                projection,
                execution: *execution,
                context,
                reads: Reads::default(),
                memory: execution
                    .reserve_retained_memory(0)
                    .map_err(|cause| Denial::Request(cause.into()))?,
                bytes: 0,
                work: 0,
                maximum_work: scope.maximum_work,
                control: scope.control,
            };
            let result = (|| {
                read.checkpoint(0)?;
                let schema = projection
                    .selected_schema_authority()
                    .filter(|_| projection.is_exact_basis())
                    .ok_or(DefinitionDenial::RecordUnavailable)?
                    .registry();
                super::super::evaluation::admit_definition(schema, scope.definition, &mut read)?;
                let result = super::super::source::acquire(
                    schema,
                    scope.definition,
                    scope.root,
                    scope.shape,
                    &mut read,
                )?;
                read.checkpoint(0)?;
                Ok::<_, Denial>(result)
            })();
            reads = read.reads;
            work = read.work;
            let (data, steps) = result?;
            Ok(PinnedDerivedRelationRead {
                data,
                steps,
                shape: scope.shape,
                work,
                _memory: read.memory,
            })
        })();
        inner_denial = result.as_ref().err().cloned();
        result
    });
    let result = match result {
        Ok((result, _)) => result,
        Err(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::NestedStopped,
        )) if inner_denial.is_some() => Err(inner_denial.expect("captured classified refusal")),
        Err(cause) => Err(Denial::Request(cause.into())),
    };
    let body = result.map_err(|kind| PinnedDerivedRelationDenial { kind, reads, work })?;
    if let Some(control) = scope.control {
        control().map_err(|cause| PinnedDerivedRelationDenial {
            kind: Denial::Request(cause),
            reads,
            work,
        })?;
    }
    execution
        .check_live()
        .map_err(|cause| PinnedDerivedRelationDenial {
            kind: Denial::Request(cause.into()),
            reads,
            work,
        })?;
    Ok(body)
}
