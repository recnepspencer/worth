//! Compile B once while preserving the genuinely readmitted A owner and roles.
//! The opaque pending custody retains B installation resources until promotion.
use super::super::super::bootstrap::native_meaning::{
    CompiledNativeSchemaMeaning, NativeSchemaCompilation, NativeSchemaInventory,
};
use super::super::super::WorthQueryBootstrapAdvancementPhase;
use super::*;
mod activation;
mod workflow_dispositions;
pub use workflow_dispositions::QueryHomeWorkflowDispositionDenial;
mod authored_body;
mod bridge;
mod program_source;
mod promotion;
pub use authored_body::WorthQueryHomeProgramTransitionBodyDenial;
mod author_source;
pub use author_source::{
    QueryHomeProgramTransitionSourceDenial, WorthQueryHomeProgramTransitionReadSource,
    WorthQueryHomeProgramTransitionTarget,
};
pub use promotion::QueryHomeTransitionPromotionRefused;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    runtime::{PreparedRelationalHomeProgramContracts, RelationalHomeProgramContractDenial},
};

pub struct PreparedHomeProgramTransitionTarget<Schema: ApplicationSchemaComposition, Program> {
    _pending: Box<HomeTransitionTargetCustody<Schema, Program>>,
    pub(super) contracts: Box<PreparedRelationalHomeProgramContracts>,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HomeTransitionTargetIndexCounts {
    pub entity_field: usize,
    pub relation_field: usize,
    pub related_ordering: usize,
    pub relation_join: usize,
}

struct HomeTransitionTargetCustody<Schema: ApplicationSchemaComposition, Program> {
    _state: Box<HomeApplicationState<Schema, Program>>,
    _layout: super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    _managed_sources:
        super::super::super::application_query::managed_index::ManagedSourceInstallation,
    _source_initializer:
        Option<super::super::super::application_query::managed_index::SourceInitializationIssuer>,
    _inventory: NativeSchemaInventory,
    index_counts: HomeTransitionTargetIndexCounts,
}
#[derive(Debug)]
pub enum QueryHomeTransitionPreparationDenial {
    Compiler(WorthQueryPrimaryGraphInstallationDenial),
    Native(RelationalHomeProgramContractDenial),
    Indexes(String),
}
pub struct QueryHomeTransitionPreparationRefused<Schema: ApplicationSchemaComposition, Program> {
    _custody: Box<TransitionPreparationCustody<Schema, Program>>,
    cause: QueryHomeTransitionPreparationDenial,
}
enum TransitionPreparationCustody<Schema: ApplicationSchemaComposition, Program> {
    Installed {
        _state: Box<HomeApplicationState<Schema, Program>>,
    },
    Compiled {
        _pending: Box<HomeTransitionTargetCustody<Schema, Program>>,
    },
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryHomeTransitionPreparationRefused<Schema, Program>
{
    pub fn reason(&self) -> &QueryHomeTransitionPreparationDenial {
        &self.cause
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
    pub fn native_contracts(&self) -> &PreparedRelationalHomeProgramContracts {
        &self.contracts
    }

    pub fn installed_index_counts(&self) -> HomeTransitionTargetIndexCounts {
        self._pending.index_counts
    }
}
impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeApplication<Schema, Program> {
    /// Compile executable B from its own declaration and A's authenticated roles.
    /// This does not install B, read A rows, publish a step or grant service.
    pub fn prepare_home_program_transition_target<
        Before: ApplicationSchemaComposition,
        BeforeProgram,
    >(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        native: &RelationalRuntime,
        predecessor: &PreparedExistingHomeSignalProgram<Before, BeforeProgram>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<
        PreparedHomeProgramTransitionTarget<Schema, Program>,
        QueryHomeTransitionPreparationRefused<Schema, Program>,
    > {
        let definition = self.home_relational_definition();
        let Self { state, invariants } = self;
        let (predecessor_layout, predecessor_receipt) = predecessor.predecessor_native_meaning();
        if predecessor_receipt.runtime_instance_id() != native.runtime_instance_id() {
            return Err(QueryHomeTransitionPreparationRefused {
                _custody: Box::new(TransitionPreparationCustody::Installed { _state: state }),
                cause: QueryHomeTransitionPreparationDenial::Native(
                    RelationalHomeProgramContractDenial::ForeignRuntime,
                ),
            });
        }
        let CompiledNativeSchemaMeaning {
            layout,
            additions,
            registrations,
            managed_sources,
            source_initializer,
        } = match state.authority.compile_native_schema(
            phase,
            &state.runtime,
            &state.installed,
            NativeSchemaCompilation::Successor {
                base: &definition.configuration().schema.registry,
                predecessor: predecessor_layout,
            },
            *invariants,
        ) {
            Ok(compiled) => compiled,
            Err(cause) => {
                return Err(QueryHomeTransitionPreparationRefused {
                    _custody: Box::new(TransitionPreparationCustody::Installed { _state: state }),
                    cause: QueryHomeTransitionPreparationDenial::Compiler(cause),
                })
            }
        };
        let mut inventory = NativeSchemaInventory::compile(&layout, &registrations);
        let sources = inventory.take_public_sources();
        let mut pending = Box::new(HomeTransitionTargetCustody {
            _state: state,
            _layout: layout,
            _managed_sources: managed_sources,
            _source_initializer: source_initializer,
            _inventory: inventory,
            index_counts: HomeTransitionTargetIndexCounts::default(),
        });
        let registry = match definition.configuration().schema.registry.clone().extend(additions) {
            Ok(registry) => registry,
            Err(cause) => return Err(QueryHomeTransitionPreparationRefused {
                _custody: Box::new(TransitionPreparationCustody::Compiled { _pending: pending }),
                cause: QueryHomeTransitionPreparationDenial::Compiler(WorthQueryPrimaryGraphInstallationDenial::new(
                    super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember, format!("successor Native registry composition refused: {cause:?}"))) }),
        };
        let execution = phase.execution_request();
        let compiled = (|| {
            let mut definitions = native
                .index_authority()
                .prepare_home_program_definitions(&execution)
                .map_err(|cause| {
                    QueryHomeTransitionPreparationDenial::Native(
                        RelationalHomeProgramContractDenial::IndexDefinitions(cause),
                    )
                })?;
            let mut native_refusal = None;
            let mut counts = HomeTransitionTargetIndexCounts::default();
            let indexes =
                super::super::super::root::index_installation::install_primary_graph_indexes(
                    &mut pending._layout,
                    |mut definition| {
                        match &definition.kind {
                            worth_relational::facade::indexes::DerivedIndexKind::EntityField {
                                ..
                            } => counts.entity_field = counts.entity_field.saturating_add(1),
                            worth_relational::facade::indexes::DerivedIndexKind::RelationField {
                                ..
                            } => counts.relation_field = counts.relation_field.saturating_add(1),
                            worth_relational::facade::indexes::DerivedIndexKind::RelatedEntityOrdering {
                                ..
                            } => {
                                counts.related_ordering =
                                    counts.related_ordering.saturating_add(1)
                            }
                            worth_relational::facade::indexes::DerivedIndexKind::RelationJoin(
                                _,
                            ) => counts.relation_join = counts.relation_join.saturating_add(1),
                        }
                        definition.index_id = definitions
                            .install(&execution, &definition)
                            .map_err(|cause| {
                                let diagnostic = format!(
                                    "successor index definition preparation refused: {cause:?}"
                                );
                                native_refusal = Some(cause);
                                diagnostic
                            })?;
                        Ok(definition)
                    },
                );
            pending.index_counts = counts;
            if let Err(diagnostic) = indexes {
                return Err(match native_refusal {
                    Some(cause) => QueryHomeTransitionPreparationDenial::Native(
                        RelationalHomeProgramContractDenial::IndexDefinitions(cause),
                    ),
                    None => QueryHomeTransitionPreparationDenial::Indexes(diagnostic),
                });
            }
            let definitions = definitions.finish(&execution).map_err(|cause| {
                QueryHomeTransitionPreparationDenial::Native(
                    RelationalHomeProgramContractDenial::IndexDefinitions(cause),
                )
            })?;
            native
                .prepare_home_program_contracts(
                    &phase.execution_request(),
                    basis,
                    registry,
                    registrations,
                    sources,
                )
                .and_then(|contracts| contracts.with_index_definitions(definitions))
                .and_then(|contracts| contracts.with_receiver_configuration(&execution, definition))
                .map_err(QueryHomeTransitionPreparationDenial::Native)
        })();
        match compiled {
            Ok(contracts) => Ok(PreparedHomeProgramTransitionTarget {
                _pending: pending,
                contracts: Box::new(contracts),
            }),
            Err(cause) => Err(QueryHomeTransitionPreparationRefused {
                _custody: Box::new(TransitionPreparationCustody::Compiled { _pending: pending }),
                cause,
            }),
        }
    }
}
