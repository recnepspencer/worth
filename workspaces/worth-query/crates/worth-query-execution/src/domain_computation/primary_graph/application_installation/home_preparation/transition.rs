//! Compile B once while preserving the genuinely readmitted A owner and roles.
//! The opaque pending custody retains B installation resources until promotion.
use super::super::super::bootstrap::native_meaning::{
    CompiledNativeSchemaMeaning, NativeSchemaCompilation, NativeSchemaInventory,
};
use super::super::super::WorthQueryBootstrapAdvancementPhase;
use super::*;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    runtime::{PreparedRelationalHomeProgramContracts, RelationalHomeProgramContractDenial},
};

pub struct PreparedHomeProgramTransitionTarget<Schema: ApplicationSchemaComposition, Program> {
    _pending: HomeTransitionTargetCustody<Schema, Program>,
    pub(super) contracts: PreparedRelationalHomeProgramContracts,
}
struct HomeTransitionTargetCustody<Schema: ApplicationSchemaComposition, Program> {
    _state: HomeApplicationState<Schema, Program>,
    _layout: super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    _managed_sources:
        super::super::super::application_query::managed_index::ManagedSourceInstallation,
    _source_initializer:
        Option<super::super::super::application_query::managed_index::SourceInitializationIssuer>,
    _inventory: NativeSchemaInventory,
}
#[derive(Debug)]
pub enum QueryHomeTransitionPreparationDenial {
    Compiler(WorthQueryPrimaryGraphInstallationDenial),
    Native(RelationalHomeProgramContractDenial),
    Indexes(String),
}
pub struct QueryHomeTransitionPreparationRefused<Schema: ApplicationSchemaComposition, Program> {
    _custody: TransitionPreparationCustody<Schema, Program>,
    cause: QueryHomeTransitionPreparationDenial,
}
enum TransitionPreparationCustody<Schema: ApplicationSchemaComposition, Program> {
    Installed {
        _state: HomeApplicationState<Schema, Program>,
    },
    Compiled {
        _pending: HomeTransitionTargetCustody<Schema, Program>,
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
        predecessor: &ReadmittedExistingHomeNativeMeaning<Before, BeforeProgram>,
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
                _custody: TransitionPreparationCustody::Installed { _state: state },
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
            invariants,
        ) {
            Ok(compiled) => compiled,
            Err(cause) => {
                return Err(QueryHomeTransitionPreparationRefused {
                    _custody: TransitionPreparationCustody::Installed { _state: state },
                    cause: QueryHomeTransitionPreparationDenial::Compiler(cause),
                })
            }
        };
        let mut inventory = NativeSchemaInventory::compile(&layout, &registrations);
        let sources = inventory.take_public_sources();
        let mut pending = HomeTransitionTargetCustody {
            _state: state,
            _layout: layout,
            _managed_sources: managed_sources,
            _source_initializer: source_initializer,
            _inventory: inventory,
        };
        let registry = match definition.configuration().schema.registry.clone().extend(additions) {
            Ok(registry) => registry,
            Err(cause) => return Err(QueryHomeTransitionPreparationRefused {
                _custody: TransitionPreparationCustody::Compiled { _pending: pending },
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
            let indexes =
                super::super::super::root::index_installation::install_primary_graph_indexes(
                    &mut pending._layout,
                    |mut definition| {
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
                contracts,
            }),
            Err(cause) => Err(QueryHomeTransitionPreparationRefused {
                _custody: TransitionPreparationCustody::Compiled { _pending: pending },
                cause,
            }),
        }
    }
}
