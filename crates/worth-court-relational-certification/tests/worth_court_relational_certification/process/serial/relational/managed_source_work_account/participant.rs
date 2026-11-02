//! The real registered Source owner adds controlled custom or SDK-funded visits.
use super::*;
use worth_relational::facade::mvcc::*;

#[derive(Debug)]
struct Refused;
impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Source provider refusal")
    }
}
impl std::error::Error for Refused {}
#[derive(Debug)]
pub(super) struct Participant {
    pub owner: owner::SourceOwner,
    pub scenario: Scenario,
    pub parent: CancellationSource,
    pub reached: Arc<AtomicUsize>,
}
impl RelationalPublicationCompanion for Participant {
    fn prepare_managed_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Vec<PreparedManagedIndexSourceRoot>, ManagedIndexSourcePreparationDenial> {
        self.reached.fetch_add(1, Ordering::SeqCst);
        let mut scopes = source.scopes();
        let scope = scopes
            .next()
            .expect("the genuine created Directory is affected");
        assert!(scopes.next().is_none());
        let rows = scope.proposed();
        let read = rows.acquire_all()?;
        assert_eq!(rows.occurrences(&read)?.len(), 1, "one actual created item");
        if self.scenario.cancel_parent {
            self.parent.cancel();
            let refused = rows
                .acquire_all()
                .err()
                .expect("parent cancellation stops the next actual Source acquisition");
            return Err(refused.into());
        }
        let roots = self.owner.prepare_managed_source(execution, source)?;
        for _ in 0..self.scenario.custom {
            scope
                .read_budget()
                .checkpoint(1)
                .map_err(DerivedRelationReadDenial::Structural)?;
        }
        let mut native = None;
        let handoff = execution.run_native_with_checkpoint(|context| {
            native = Some((|| {
                for _ in 0..self.scenario.native {
                    scope.read_budget().checkpoint_native(context, 1).map_err(
                        |cause| match cause {
                            CustomInvariantNativeCheckpointDenial::Source(cause) => {
                                DerivedRelationReadDenial::Structural(cause)
                            }
                            CustomInvariantNativeCheckpointDenial::Request(stop) => {
                                DerivedRelationReadDenial::Request(
                                    RelationalExecutionDenialCause::from(
                                        WorkCeilingDenial::Stopped(stop),
                                    ),
                                )
                            }
                        },
                    )?;
                }
                Ok::<_, DerivedRelationReadDenial>(())
            })());
        });
        match native {
            Some(Err(cause)) => return Err(cause.into()),
            Some(Ok(())) => {
                handoff.map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?
            }
            None => {
                return Err(DerivedRelationReadDenial::Request(handoff.unwrap_err().into()).into())
            }
        };
        if self.scenario.refuse {
            return Err(ManagedIndexSourcePreparationDenial::Provider(
                ManagedIndexSourceProviderFailure::from_owned(Arc::new(Refused)),
            ));
        }
        Ok(roots)
    }
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        self.owner.prepare(context)
    }
}
pub(super) fn is_refused(error: &ManagedIndexSourcePreparationDenial) -> bool {
    error
        .provider_failure()
        .is_some_and(|failure| failure.cause().is::<Refused>())
}
