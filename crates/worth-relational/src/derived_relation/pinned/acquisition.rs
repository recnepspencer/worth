//! One controlled acquisition owner serves both old and receipt-bearing doors.
use super::*;

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub fn derived_relations_from(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
        maximum_work: Option<usize>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        self.derived_source(
            execution,
            definition,
            root,
            super::super::DerivedRelationReadShape::Adjacency { reverse: false },
            maximum_work,
        )
    }
    /// Evaluate the exact registered recipe under the original request.
    pub fn derived_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
        shape: super::super::DerivedRelationReadShape,
        maximum_work: Option<usize>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        self.read_derived_source(
            execution,
            PinnedDerivedRelationScope::new(definition, root, shape).with_work_limit(maximum_work),
        )
        .map_err(|stop| stop.kind)
    }
    /// Complete data retains its actual payer. Refusal retains actual reads/work
    /// without returning any partial successful data or replacement allowance.
    pub fn read_derived_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedDerivedRelationScope<'_>,
    ) -> Result<PinnedDerivedRelationRead, PinnedDerivedRelationDenial> {
        VisibilityProjectionView::from_pinned(self).read_derived_source(execution, scope)
    }
}
impl VisibilityProjectionView {
    pub(crate) fn derived_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        definition: &Handle,
        root: NativeMemberKey,
        shape: super::super::DerivedRelationReadShape,
        maximum_work: Option<usize>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        self.read_derived_source(
            execution,
            PinnedDerivedRelationScope::new(definition, root, shape).with_work_limit(maximum_work),
        )
        .map_err(|stop| stop.kind)
    }
    fn read_derived_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedDerivedRelationScope<'_>,
    ) -> Result<PinnedDerivedRelationRead, PinnedDerivedRelationDenial> {
        let mut reads = Reads::default();
        let mut work = 0;
        let mut inner_denial = None;
        let result = execution.run_native_with_checkpoint(|context| {
            let result = (|| {
                let mut read = PinnedSource {
                    projection: self,
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
                    let schema = self
                        .selected_schema_authority()
                        .filter(|_| self.is_exact_basis())
                        .ok_or(DefinitionDenial::RecordUnavailable)?
                        .registry();
                    super::super::evaluation::admit_definition(
                        schema,
                        scope.definition,
                        &mut read,
                    )?;
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
}
