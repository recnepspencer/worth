//! The cell model uses the ordinary program, bootstrap and authentication owners.
use super::{edit, schema::*};
use crate::{document_world_resources::world_resources, operator_identity::operator_identity};
use worth_query_decl::facade::{application_program::*, authentication::*};
use worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};

pub struct Program;
pub struct CellAuthoring;
impl ApplicationFeature<Schema> for CellAuthoring {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "cell-history.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("cell-history.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, CellAuthoring>()
            .native_action::<edit::Binding>()
            .finish_action()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<edit::Binding, _>(edit::Handler)
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

pub fn install() -> Host {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("16161616-1616-1616-1616-161616161616").unwrap(),
        WorthQueryInMemoryApplicationLimits::new(
            world_resources(64, 1_048_576, 128),
            WorthQueryApplicationQueryResourceProfile::new(
                std::num::NonZeroUsize::new(32).unwrap(),
            ),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            graph.bind_principal(
                &installed
                    .principal_binding(PrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("cell-author").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for feature in [1_u64, 2] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Feature::reference(),
                        WorthQueryApplicationEntityKey::new(format!("feature-{feature}")).unwrap(),
                    )
                    .field(FeatureKey::reference(), feature),
                )?;
            }
            for (cell, feature, value) in [(10_u64, 1_u64, 7_u64), (11, 1, 13), (20, 2, 29)] {
                let cell_key = WorthQueryApplicationEntityKey::new(format!("cell-{cell}")).unwrap();
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Cell::reference(),
                        WorthQueryApplicationEntityKey::new(format!("cell-{cell}")).unwrap(),
                    )
                    .field(CellKey::reference(), cell)
                    .field(CellValue::reference(), value),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    OwnsCell::reference(),
                    format!("owns-{cell}"),
                    WorthQueryApplicationEntityKey::new(format!("feature-{feature}")).unwrap(),
                    cell_key,
                ))?;
            }
            Ok(())
        },
    )
    .expect("the ordinary cell-bearing application installs")
}
