//! Prepare the exact opening custody before entering any physical close frame.
use super::*;

pub(super) enum OpeningClosePlan<Schema: ApplicationSchemaComposition, Program> {
    NoHome,
    RelationalAttachment(Box<RelationalAttachmentRefused>),
    Unrecovered(Box<UnrecoveredApplicationHome>),
    Unshared(Box<UnsharedReadmittedApplicationHome>),
    NativeReadmitted(Box<ReadmittedApplicationHome>),
    WorldReadmission(Box<WorldHomeReadmissionRefused>),
    World {
        query: Box<WorldQueryPreparation<Schema, Program>>,
        home: Box<DirectoryHome>,
    },
}

#[inline(never)]
pub(super) fn prepare<Schema: ApplicationSchemaComposition, Program: 'static>(
    source: Box<WorthQueryApplicationHomeOpenRefusedPhase<Schema, Program>>,
) -> OpeningClosePlan<Schema, Program> {
    let source = match prepare_pre_world(source) {
        Ok(plan) => return plan,
        Err(source) => source,
    };
    let source = match prepare_existing_and_transition_world(source) {
        Ok(plan) => return plan,
        Err(source) => source,
    };
    prepare_fresh_world(source)
}

#[inline(never)]
fn prepare_pre_world<Schema: ApplicationSchemaComposition, Program: 'static>(
    source: Box<WorthQueryApplicationHomeOpenRefusedPhase<Schema, Program>>,
) -> Result<
    OpeningClosePlan<Schema, Program>,
    Box<WorthQueryApplicationHomeOpenRefusedPhase<Schema, Program>>,
> {
    use WorthQueryApplicationHomeOpenRefusedPhase::*;
    Ok(match *source {
        Preparation(_) => OpeningClosePlan::NoHome,
        Binding { query, cause: _ } | Physical { query, cause: _ } => {
            drop(query);
            OpeningClosePlan::NoHome
        }
        SignalProgramPreparation { home, query } => {
            drop(query);
            OpeningClosePlan::NativeReadmitted(Box::new(home.into_home()))
        }
        WorldSourcePreparation { query, home } => {
            drop(query);
            OpeningClosePlan::WorldReadmission(Box::new(home))
        }
        ExistingWorldSourcePreparation { query, home } => {
            drop(query);
            OpeningClosePlan::WorldReadmission(Box::new(home))
        }
        RelationalAttachment { query, home } => {
            drop(query);
            OpeningClosePlan::RelationalAttachment(Box::new(home))
        }
        NativePreparation {
            query,
            home,
            cause: _,
        } => {
            drop(query);
            OpeningClosePlan::Unrecovered(Box::new(home))
        }
        NativeInstallation { query, home } => {
            // These unshared meanings contain no Native/source handle.
            // Explicit disposal releases them before the real raw close.
            drop(query);
            OpeningClosePlan::Unrecovered(Box::new(home))
        }
        NativeReadmission { query, home } => {
            drop(query);
            OpeningClosePlan::Unrecovered(Box::new(home.into_home()))
        }
        QueryPublicationReadmissionRequired { query, home } => {
            drop(query);
            OpeningClosePlan::Unshared(Box::new(home.into_home()))
        }
        NativeFinalizationInput {
            query,
            home,
            cause: _,
        } => {
            drop(query);
            OpeningClosePlan::Unshared(Box::new(home))
        }
        ExistingNativeFinalization { query, home } => {
            drop(query);
            OpeningClosePlan::Unshared(Box::new(home))
        }
        ExistingBridgeDefinition {
            query,
            home,
            cause: _,
        } => {
            drop(query);
            OpeningClosePlan::Unshared(Box::new(home))
        }
        ExistingWorldReadmission {
            query,
            home,
            _bindings,
        } => {
            drop(_bindings);
            drop(query);
            OpeningClosePlan::WorldReadmission(Box::new(home))
        }
        NativeFinalization { query, home } => {
            drop(query);
            OpeningClosePlan::Unshared(Box::new(home))
        }
        BridgeDefinition {
            query,
            home,
            cause: _,
        } => {
            drop(query);
            OpeningClosePlan::Unshared(Box::new(home))
        }
        WorldReadmission {
            query,
            home,
            _bindings,
        } => {
            drop(_bindings);
            drop(query);
            // Store's owner method releases its failed, unshared World
            // composition before returning the actual Native Home.
            OpeningClosePlan::WorldReadmission(Box::new(home))
        }
        other => return Err(Box::new(other)),
    })
}

#[inline(never)]
fn prepare_existing_and_transition_world<Schema: ApplicationSchemaComposition, Program: 'static>(
    source: Box<WorthQueryApplicationHomeOpenRefusedPhase<Schema, Program>>,
) -> Result<
    OpeningClosePlan<Schema, Program>,
    Box<WorthQueryApplicationHomeOpenRefusedPhase<Schema, Program>>,
> {
    use WorthQueryApplicationHomeOpenRefusedPhase::*;
    Ok(match *source {
        TransitionComposition { query, home, cause } => world(
            WorldQueryPreparation::TransitionComposition {
                _query: query,
                _cause: cause,
            },
            home,
        ),
        TransitionSignalProgramPreparation { query, home } => world(
            WorldQueryPreparation::TransitionSignal { _query: query },
            home,
        ),
        TransitionProgrammeHandoff {
            query,
            home,
            handoff,
            _bindings,
        } => world(
            WorldQueryPreparation::TransitionProgramme {
                _query: query,
                _handoff: handoff,
                _bindings,
            },
            home,
        ),
        ExistingComposition { query, home, cause } => world(
            WorldQueryPreparation::ExistingComposition {
                _query: query,
                _cause: cause,
            },
            home,
        ),
        ExistingSourceAttachment { query, home } => {
            world(WorldQueryPreparation::ExistingSource(query), home)
        }
        ExistingCheckpointMaintenance { query, home, cause } => world(
            WorldQueryPreparation::ExistingMaintenance {
                _query: query,
                _cause: cause,
            },
            home,
        ),
        ExistingWorldRoot { query, home, cause } => world(
            WorldQueryPreparation::ExistingRoot {
                _query: query,
                _cause: cause,
            },
            home,
        ),
        ExistingQueryAttachment { query, home } => {
            world(WorldQueryPreparation::ExistingAttachment(query), home)
        }
        other => return Err(Box::new(other)),
    })
}

#[inline(never)]
fn prepare_fresh_world<Schema: ApplicationSchemaComposition, Program: 'static>(
    source: Box<WorthQueryApplicationHomeOpenRefusedPhase<Schema, Program>>,
) -> OpeningClosePlan<Schema, Program> {
    use WorthQueryApplicationHomeOpenRefusedPhase::*;
    match *source {
        Composition { query, home, cause } => world(
            WorldQueryPreparation::Composition {
                _query: query,
                _cause: cause,
            },
            home,
        ),
        SourceAttachment { query, home } => world(WorldQueryPreparation::Source(query), home),
        InitialState { query, home, cause } => world(
            WorldQueryPreparation::Bootstrap {
                _query: query,
                _cause: BootstrapFailure::InitialState(cause),
            },
            home,
        ),
        SeedPublication { query, home, cause } => world(
            WorldQueryPreparation::Bootstrap {
                _query: query,
                _cause: BootstrapFailure::Seed(cause),
            },
            home,
        ),
        CheckpointMaintenance { query, home, cause } => world(
            WorldQueryPreparation::Bootstrap {
                _query: query,
                _cause: BootstrapFailure::Maintenance(cause),
            },
            home,
        ),
        WorldRoot { query, home, cause } => world(
            WorldQueryPreparation::Bootstrap {
                _query: query,
                _cause: BootstrapFailure::Root(cause),
            },
            home,
        ),
        QueryAttachment { query, home } => world(WorldQueryPreparation::Attachment(query), home),
        CustodyReadmission { query, home, cause } => world(
            WorldQueryPreparation::Custody {
                _query: query,
                _cause: cause,
            },
            home,
        ),
        OwnerAdmission { query, home } => {
            let (home, cause) = home.into_parts();
            world(
                WorldQueryPreparation::OwnerAdmission {
                    _query: query,
                    _cause: cause,
                },
                home,
            )
        }
        HistoricalBranchReconciliation { query, home, cause } => world(
            WorldQueryPreparation::HistoricalBranchReconciliation {
                _query: query,
                _cause: cause,
            },
            home,
        ),
        _ => unreachable!("pre-world, transition, and existing variants are handled earlier"),
    }
}

#[inline(never)]
fn world<Schema: ApplicationSchemaComposition, Program>(
    query: WorldQueryPreparation<Schema, Program>,
    home: DirectoryHome,
) -> OpeningClosePlan<Schema, Program> {
    OpeningClosePlan::World {
        query: Box::new(query),
        home: Box::new(home),
    }
}

impl<Schema: ApplicationSchemaComposition, Program: 'static> OpeningClosePlan<Schema, Program> {
    #[inline(never)]
    pub(super) fn execute(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryApplicationHomeOpeningClosed,
        WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
    > {
        match self {
            Self::NoHome => Ok(WorthQueryApplicationHomeOpeningClosed {
                outcome: None,
                retired_query: None,
            }),
            Self::RelationalAttachment(home) => {
                let outcome = match *home {
                    RelationalAttachmentRefused::BeforeConstruction { home, cause: _ } => {
                        home.consume(|home| home.close(request))
                    }
                    RelationalAttachmentRefused::Construction(home) => home.close(request).1,
                };
                classify_released(outcome)
            }
            Self::Unrecovered(home) => close_unrecovered(*home, request),
            Self::Unshared(home) => close_unshared(*home, request),
            Self::NativeReadmitted(home) => classify_released((*home).close(request)),
            Self::WorldReadmission(home) => classify_released((*home).into_home().close(request)),
            Self::World { query, home } => close_world_preparation(query, *home, request),
        }
    }
}
