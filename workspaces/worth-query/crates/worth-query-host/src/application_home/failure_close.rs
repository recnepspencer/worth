//! Explicit disposal keeps each actual Home close phase on refusal.
use super::{close::*, refusal::*, runtime::DirectoryHome};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_home_integration::*, primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
use worth_store_application::*;

pub struct WorthQueryApplicationHomeOpeningClosed {
    outcome: Option<ApplicationHomeCloseOutcome>,
}
impl WorthQueryApplicationHomeOpeningClosed {
    pub fn directory_outcome(&self) -> Option<&ApplicationHomeCloseOutcome> {
        self.outcome.as_ref()
    }
}
pub struct WorthQueryApplicationHomeOpeningCloseRefused<
    Schema: ApplicationSchemaComposition,
    Program,
> {
    custody: OpeningCloseCustody<Schema, Program>,
}
enum OpeningCloseCustody<Schema: ApplicationSchemaComposition, Program> {
    Unrecovered(UnrecoveredApplicationHomeCloseRefused),
    Unshared(UnsharedApplicationHomeCloseRefused),
    World {
        query: WorldQueryPreparation<Schema, Program>,
        home: WorldHomeCloseRefused<(), (), (), (), ()>,
    },
    ReleasedQuery(WorthQueryApplicationHomeCloseRefused<Schema, Program>),
}
enum WorldQueryPreparation<Schema: ApplicationSchemaComposition, Program> {
    Composition {
        _query: ReadmittedHomeNativeMeaning<Schema, Program>,
        _cause: ApplicationHomeQueryCompositionDenial,
    },
    Source(QueryHomeSourceAttachmentRefused<Schema, Program>),
    Bootstrap {
        _query: PreparedHomeQueryRuntime<Schema, Program>,
        _cause: BootstrapFailure,
    },
    Attachment(QueryHomeWorldAttachmentRefused<Schema, Program>),
    Custody {
        _query: ReadmittedHomeQueryService<Schema, Program>,
        _cause: HomeCustodyReadmissionDenial,
    },
    OwnerAdmission {
        _query: ReadmittedHomeQueryService<Schema, Program>,
        _cause: ApplicationHomeOwnerAdmissionDenial,
    },
}
enum BootstrapFailure {
    InitialState(QueryHomeInitialStateDenial),
    Seed(WorthQueryPrimaryGraphInstallationDenial),
    Root(ApplicationHomeQueryRootDenial),
}
impl BootstrapFailure {
    fn detail(&self) -> String {
        match self {
            Self::InitialState(cause) => format!("{cause:?}"),
            Self::Seed(cause) => cause.to_string(),
            Self::Root(cause) => format!("{cause:?}"),
        }
    }
}
impl<Schema: ApplicationSchemaComposition, Program> WorldQueryPreparation<Schema, Program> {
    fn detail(&self) -> String {
        match self {
            Self::Composition { _cause, .. } => format!("{_cause:?}"),
            Self::Source(query) => query.reason().to_string(),
            Self::Bootstrap { _cause, .. } => _cause.detail(),
            Self::Attachment(query) => query.detail(),
            Self::Custody { _cause, .. } => format!("{_cause:?}"),
            Self::OwnerAdmission { _cause, .. } => format!("{_cause:?}"),
        }
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    pub fn close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryApplicationHomeOpeningClosed,
        WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
    > {
        use WorthQueryApplicationHomeOpenRefused::*;
        match self {
            Preparation(_) => Ok(WorthQueryApplicationHomeOpeningClosed { outcome: None }),
            Binding { query, cause: _ } | Physical { query, cause: _ } => {
                drop(query);
                Ok(WorthQueryApplicationHomeOpeningClosed { outcome: None })
            }
            RelationalAttachment { query, home } => {
                drop(query);
                let outcome = match home {
                    RelationalAttachmentRefused::BeforeConstruction { home, cause: _ } => {
                        home.consume(|home| home.close(request))
                    }
                    RelationalAttachmentRefused::Construction(home) => home.close(request).1,
                };
                classify_released(outcome)
            }
            NativeInstallation { query, home } => {
                // These unshared meanings contain no Native/source handle.
                // Explicit disposal releases them before the real raw close.
                drop(query);
                close_unrecovered(home, request)
            }
            NativeReadmission { query, home } => {
                drop(query);
                close_unrecovered(home.into_home(), request)
            }
            QueryPublicationReadmissionRequired { query, home } => {
                drop(query);
                close_unshared(home.into_home(), request)
            }
            NativeFinalization { query, home } => {
                drop(query);
                close_unshared(home, request)
            }
            BridgeDefinition {
                query,
                home,
                cause: _,
            } => {
                drop(query);
                close_unshared(home, request)
            }
            WorldReadmission { query, home } => {
                drop(query);
                // Store's owner method releases its failed, unshared World
                // composition before returning the actual Native Home.
                classify_released(home.into_home().close(request))
            }
            Composition { query, home, cause } => close_world_preparation(
                WorldQueryPreparation::Composition {
                    _query: query,
                    _cause: cause,
                },
                home,
                request,
            ),
            SourceAttachment { query, home } => {
                close_world_preparation(WorldQueryPreparation::Source(query), home, request)
            }
            InitialState { query, home, cause } => close_world_preparation(
                WorldQueryPreparation::Bootstrap {
                    _query: query,
                    _cause: BootstrapFailure::InitialState(cause),
                },
                home,
                request,
            ),
            SeedPublication { query, home, cause } => close_world_preparation(
                WorldQueryPreparation::Bootstrap {
                    _query: query,
                    _cause: BootstrapFailure::Seed(cause),
                },
                home,
                request,
            ),
            WorldRoot { query, home, cause } => close_world_preparation(
                WorldQueryPreparation::Bootstrap {
                    _query: query,
                    _cause: BootstrapFailure::Root(cause),
                },
                home,
                request,
            ),
            QueryAttachment { query, home } => {
                close_world_preparation(WorldQueryPreparation::Attachment(query), home, request)
            }
            CustodyReadmission { query, home, cause } => close_world_preparation(
                WorldQueryPreparation::Custody {
                    _query: query,
                    _cause: cause,
                },
                home,
                request,
            ),
            OwnerAdmission { query, home } => {
                let (home, cause) = home.into_parts();
                close_world_preparation(
                    WorldQueryPreparation::OwnerAdmission {
                        _query: query,
                        _cause: cause,
                    },
                    home,
                    request,
                )
            }
        }
    }
}
fn close_unrecovered<Schema: ApplicationSchemaComposition, Program>(
    home: UnrecoveredApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    match home.close(request) {
        Ok(outcome) => classify_released(outcome),
        Err(home) => Err(WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::Unrecovered(home),
        }),
    }
}
fn close_unshared<Schema: ApplicationSchemaComposition, Program>(
    home: UnsharedReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    match home.close(request) {
        Ok(outcome) => classify_released(outcome),
        Err(home) => Err(WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::Unshared(home),
        }),
    }
}
fn close_world_preparation<Schema: ApplicationSchemaComposition, Program>(
    query: WorldQueryPreparation<Schema, Program>,
    home: DirectoryHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    match home.seal_world(request) {
        Ok(home) => {
            drop(query);
            classify_released(home.finish_close(request))
        }
        Err(home) => Err(WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::World { query, home },
        }),
    }
}
fn classify_released<Schema: ApplicationSchemaComposition, Program>(
    outcome: ApplicationHomeCloseOutcome,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    super::close::classify_released_query(outcome)
        .map(|closed| WorthQueryApplicationHomeOpeningClosed {
            outcome: Some(closed.into_outcome()),
        })
        .map_err(|home| WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::ReleasedQuery(home),
        })
}
impl<Schema: ApplicationSchemaComposition, Program> std::fmt::Debug
    for WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.custody {
            OpeningCloseCustody::Unrecovered(home) => {
                f.debug_tuple("Unrecovered").field(home.reason()).finish()
            }
            OpeningCloseCustody::Unshared(home) => {
                f.debug_tuple("Unshared").field(home.reason()).finish()
            }
            OpeningCloseCustody::World { query, home } => f
                .debug_struct("World")
                .field("opening", &query.detail())
                .field("close", home.reason())
                .finish(),
            OpeningCloseCustody::ReleasedQuery(home) => home.fmt(f),
        }
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>
{
    pub fn try_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeOpeningClosed, Self> {
        match self.custody {
            OpeningCloseCustody::Unrecovered(home) => match home.try_close(request) {
                Ok(outcome) => classify_released(outcome),
                Err(home) => Err(Self {
                    custody: OpeningCloseCustody::Unrecovered(home),
                }),
            },
            OpeningCloseCustody::Unshared(home) => match home.try_close(request) {
                Ok(outcome) => classify_released(outcome),
                Err(home) => Err(Self {
                    custody: OpeningCloseCustody::Unshared(home),
                }),
            },
            OpeningCloseCustody::World { query, home } => match home.try_seal_world(request) {
                Ok(home) => {
                    drop(query);
                    classify_released(home.finish_close(request))
                }
                Err(home) => Err(Self {
                    custody: OpeningCloseCustody::World { query, home },
                }),
            },
            OpeningCloseCustody::ReleasedQuery(home) => home
                .try_close(request)
                .map(|closed| WorthQueryApplicationHomeOpeningClosed {
                    outcome: Some(closed.into_outcome()),
                })
                .map_err(|home| Self {
                    custody: OpeningCloseCustody::ReleasedQuery(home),
                }),
        }
    }
}
