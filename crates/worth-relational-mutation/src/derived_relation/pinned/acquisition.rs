//! One controlled acquisition owner serves both old and receipt-bearing doors.
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::view::VisibilityProjectionViewOwnerPort as _;
use super::*;
use crate::contracts::derived_relation::read_shape::DerivedRelationReadShape;

impl PinnedDerivedRelationRead {
    pub(crate) fn acquire_projected(
        projection: &VisibilityProjectionView,
        execution: &ExecutionRequest<'_, '_>,
        definition: &Handle,
        root: NativeMemberKey,
        shape: worth_relational_contracts::internal_port::derived_relation::read_shape::DerivedRelationReadShape,
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

pub fn read_projected_source(
    projection: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    scope: PinnedDerivedRelationScope<'_>,
) -> Result<PinnedDerivedRelationRead, PinnedDerivedRelationDenial> {
    let shape = scope.shape;
    let (data, steps, work, memory) =
        match run_projected_source(projection, execution, scope, Output::Facts)? {
            Acquired::Facts {
                data,
                steps,
                work,
                memory,
            } => (data, steps, work, memory),
            Acquired::Count(_) => unreachable!("fact acquisition was requested"),
        };
    Ok(PinnedDerivedRelationRead {
        data,
        steps,
        shape,
        work,
        _memory: memory,
    })
}

pub fn count_projected_adjacency(
    projection: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    definition: &Handle,
    root: NativeMemberKey,
    maximum_work: Option<usize>,
) -> Result<usize, PinnedDerivedRelationDenial> {
    let scope = PinnedDerivedRelationScope::new(
        definition,
        root,
        DerivedRelationReadShape::Adjacency { reverse: false },
    )
    .with_work_limit(maximum_work);
    match run_projected_source(projection, execution, scope, Output::Count)? {
        Acquired::Count(count) => Ok(count),
        Acquired::Facts { .. } => unreachable!("scalar acquisition was requested"),
    }
}

enum Output {
    Facts,
    Count,
}

enum Acquired {
    Facts {
        data: Data,
        steps: Vec<crate::derived_relation::proof::DerivedRelationWalkStep>,
        work: usize,
        memory: Option<ExecutionMemoryReservation>,
    },
    Count(usize),
}

fn run_projected_source(
    projection: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    scope: PinnedDerivedRelationScope<'_>,
    output: Output,
) -> Result<Acquired, PinnedDerivedRelationDenial> {
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
                    .selected_schema()
                    .filter(|_| projection.is_exact_basis())
                    .ok_or(DefinitionDenial::RecordUnavailable)?
                    .registry;
                super::super::evaluation::admit_definition(schema, scope.definition, &mut read)?;
                match output {
                    Output::Facts => {
                        let (data, steps) = super::super::source::acquire(
                            schema,
                            scope.definition,
                            scope.root,
                            scope.shape,
                            &mut read,
                        )?;
                        read.checkpoint(0)?;
                        Ok(Acquired::Facts {
                            data,
                            steps,
                            work: read.work,
                            memory: read.memory.take(),
                        })
                    }
                    Output::Count => {
                        let count = super::super::evaluation::count_position(
                            schema,
                            scope.definition,
                            scope.definition.parts().nodes.len() - 1,
                            false,
                            scope.root,
                            &mut read,
                        )?;
                        read.checkpoint(0)?;
                        Ok(Acquired::Count(count))
                    }
                }
            })();
            reads = read.reads;
            work = read.work;
            result
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
