//! Prepare the exact opening custody before entering any physical close frame.
use super::*;

pub(super) enum OpeningClosePlan<Schema: ApplicationSchemaComposition, Program> {
    NoHome,
    RelationalAttachment(RelationalAttachmentRefused),
    Unrecovered(UnrecoveredApplicationHome),
    Unshared(UnsharedReadmittedApplicationHome),
    WorldReadmission(WorldHomeReadmissionRefused),
    World {
        query: WorldQueryPreparation<Schema, Program>,
        home: DirectoryHome,
    },
}

#[inline(never)]
pub(super) fn prepare<Schema: ApplicationSchemaComposition, Program: 'static>(
    source: WorthQueryApplicationHomeOpenRefused<Schema, Program>,
) -> OpeningClosePlan<Schema, Program> {
    use WorthQueryApplicationHomeOpenRefused::*;
    match source {
        Preparation(_) => OpeningClosePlan::NoHome,
        Binding { query, cause: _ } | Physical { query, cause: _ } => {
            drop(query);
            OpeningClosePlan::NoHome
        }
        RelationalAttachment { query, home } => {
            drop(query);
            OpeningClosePlan::RelationalAttachment(home)
        }
        NativeInstallation { query, home } => {
            // These unshared meanings contain no Native/source handle.
            // Explicit disposal releases them before the real raw close.
            drop(query);
            OpeningClosePlan::Unrecovered(home)
        }
        NativeReadmission { query, home } => {
            drop(query);
            OpeningClosePlan::Unrecovered(home.into_home())
        }
        QueryPublicationReadmissionRequired { query, home } => {
            drop(query);
            OpeningClosePlan::Unshared(home.into_home())
        }
        NativeFinalizationInput {
            query,
            home,
            cause: _,
        } => {
            drop(query);
            OpeningClosePlan::Unshared(home)
        }
        ExistingNativeFinalization { query, home } => {
            drop(query);
            OpeningClosePlan::Unshared(home)
        }
        ExistingBridgeDefinition {
            query,
            home,
            cause: _,
        } => {
            drop(query);
            OpeningClosePlan::Unshared(home)
        }
        ExistingWorldReadmission { query, home } => {
            drop(query);
            OpeningClosePlan::WorldReadmission(home)
        }
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
        NativeFinalization { query, home } => {
            drop(query);
            OpeningClosePlan::Unshared(home)
        }
        BridgeDefinition {
            query,
            home,
            cause: _,
        } => {
            drop(query);
            OpeningClosePlan::Unshared(home)
        }
        WorldReadmission { query, home } => {
            drop(query);
            // Store's owner method releases its failed, unshared World
            // composition before returning the actual Native Home.
            OpeningClosePlan::WorldReadmission(home)
        }
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
    }
}

fn world<Schema: ApplicationSchemaComposition, Program>(
    query: WorldQueryPreparation<Schema, Program>,
    home: DirectoryHome,
) -> OpeningClosePlan<Schema, Program> {
    OpeningClosePlan::World { query, home }
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
                let outcome = match home {
                    RelationalAttachmentRefused::BeforeConstruction { home, cause: _ } => {
                        home.consume(|home| home.close(request))
                    }
                    RelationalAttachmentRefused::Construction(home) => home.close(request).1,
                };
                classify_released(outcome)
            }
            Self::Unrecovered(home) => close_unrecovered(home, request),
            Self::Unshared(home) => close_unshared(home, request),
            Self::WorldReadmission(home) => classify_released(home.into_home().close(request)),
            Self::World { query, home } => close_world_preparation(query, home, request),
        }
    }
}
