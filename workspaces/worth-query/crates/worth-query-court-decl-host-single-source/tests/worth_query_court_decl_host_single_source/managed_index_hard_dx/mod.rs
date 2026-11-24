//! Managed-Index Hard DX acceptance courts and 64-seed 4-family differential DST.
mod courts;
mod dst;
mod dst_driver;
mod dst_oracle;
mod harness;
mod operations;
mod schema;

use harness::*;
use operations::*;
use schema::*;
use worth_query_decl::facade::application_query::{
    ApplicationManagedIndexDefinition, ApplicationQueryOrderingDirection, ApplicationQueryRootPath,
};
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::ordered_index::{
    WorthQueryBox2DIndex, WorthQueryBox3DIndex, WorthQueryInterval1DIndex,
    WorthQueryTextPrefixIndex,
};
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
    WorthQueryApplicationHome,
};

pub struct ShopFeature;
impl ApplicationFeature<ShopFloorSchema> for ShopFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "hard_dx.shop.feature.v1";
}

pub struct Contracts;
impl Registration<ShopFloorSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<ShopFloorSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<ShopFloorSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let asc = ApplicationQueryOrderingDirection::Ascending;
        Ok(schema
            .principal_binding(PrincipalBinding::reference())
            .application_query(
                AssetSourceQuery::definition()
                    .root_path(
                        ApplicationQueryRootPath::from(Floor::reference())
                            .forward(ContainsAsset::reference()),
                    )
                    .order_by(AssetRow::key(), asc),
            )
            .application_query_binding::<AssetSourceBinding>()
            .application_query(FindByTitlePrefixQuery::definition().order_by(AssetRow::key(), asc))
            .application_query_binding::<FindByTitlePrefixBinding>()
            .application_query(FindByFocusWindowQuery::definition().order_by(AssetRow::key(), asc))
            .application_query_binding::<FindByFocusWindowBinding>()
            .application_query(
                FindEquipmentInBox2DQuery::definition().order_by(AssetRow::key(), asc),
            )
            .application_query_binding::<FindEquipmentInBox2DBinding>()
            .application_query(FindRoutesInBox3DQuery::definition().order_by(AssetRow::key(), asc))
            .application_query_binding::<FindRoutesInBox3DBinding>()
            .application_query(FindInFloatBox2DQuery::definition().order_by(AssetRow::key(), asc))
            .application_query_binding::<FindInFloatBox2DBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<
                ShopFloorSchema,
                TitlePrefixIndex,
            >::new())
            .managed_index(ApplicationManagedIndexDefinition::<
                ShopFloorSchema,
                FocusIntervalIndex,
            >::new())
            .managed_index(ApplicationManagedIndexDefinition::<
                ShopFloorSchema,
                EquipmentBox2DIndex,
            >::new())
            .managed_index(ApplicationManagedIndexDefinition::<
                ShopFloorSchema,
                RouteBox3DIndex,
            >::new())
            .managed_index(ApplicationManagedIndexDefinition::<
                ShopFloorSchema,
                FloatBox2DIndex,
            >::new())
            .declared_operation::<CreateAssetOp>(
                CreateAssetOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<CreateAssetBinding>()
            .declared_operation::<UpdateAssetOp>(
                UpdateAssetOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<UpdateAssetBinding>()
            .declared_operation::<DeleteAssetOp>(
                DeleteAssetOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<DeleteAssetBinding>())
    }
}

pub struct Contribution;
impl ApplicationSchemaContribution<ShopFloorSchema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("hard_dx.shop.contribution.v1");
    type Members = (ShopFloor, Contracts);
}
impl WorthQueryApplicationContribution<ShopFloorSchema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, ShopFloorSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup
            .managed_index_provider::<TitlePrefixIndex, _>(
                WorthQueryTextPrefixIndex::ascii_case_insensitive(AssetRow::title()),
            )
            .with_predicate_routing()
            .install()?;
        setup
            .managed_index_provider::<FocusIntervalIndex, _>(WorthQueryInterval1DIndex::half_open(
                AssetRow::start_min(),
                AssetRow::end_min(),
            ))
            .with_predicate_routing()
            .install()?;
        setup
            .managed_index_provider::<EquipmentBox2DIndex, _>(WorthQueryBox2DIndex::half_open(
                AssetRow::min_x(),
                AssetRow::min_y(),
                AssetRow::max_x(),
                AssetRow::max_y(),
            ))
            .with_predicate_routing()
            .install()?;
        setup
            .managed_index_provider::<RouteBox3DIndex, _>(WorthQueryBox3DIndex::closed(
                (AssetRow::min_x(), AssetRow::min_y(), AssetRow::min_z()),
                (AssetRow::max_x(), AssetRow::max_y(), AssetRow::max_z()),
            ))
            .with_predicate_routing()
            .install()?;
        setup
            .managed_index_provider::<FloatBox2DIndex, _>(WorthQueryBox2DIndex::half_open(
                AssetRow::fmin_x(),
                AssetRow::fmin_y(),
                AssetRow::fmax_x(),
                AssetRow::fmax_y(),
            ))
            .with_predicate_routing()
            .install()?;
        setup.handler::<CreateAssetBinding, _>(CreateAssetHandler)?;
        setup.handler::<UpdateAssetBinding, _>(UpdateAssetHandler)?;
        setup.handler::<DeleteAssetBinding, _>(DeleteAssetHandler)
    }
}

pub struct Program;
impl ApplicationProgramDefinition<ShopFloorSchema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("hard_dx.shop.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ShopFloorSchema, ShopFeature>()
                .native_action::<CreateAssetBinding>()
                .finish_action()
                .native_action::<UpdateAssetBinding>()
                .finish_action()
                .native_action::<DeleteAssetBinding>()
                .finish_action()
                .finish(),
        ]
    }
}

pub(super) type Host = WorthQueryProgramApplicationRuntime<ShopFloorSchema, Program>;
pub(super) type Home = WorthQueryApplicationHome<ShopFloorSchema, Program>;
