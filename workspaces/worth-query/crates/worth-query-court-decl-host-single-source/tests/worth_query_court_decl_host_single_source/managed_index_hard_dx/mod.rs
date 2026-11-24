//! Managed-Index Hard DX acceptance courts and 64-seed 4-family differential DST.
//! Wired end-to-end through Target DX #7 (`#[worth_binary]`, `#[worth_index(..., kind = ..)]`,
//! and `worth_query_application_program!` `managed_indexes: [...]`).
mod courts;
mod courts_f64;
mod dst;
mod dst_driver;
mod dst_oracle;
mod harness;
mod operations;
mod schema;

use harness::*;
use operations::*;
use schema::*;
use worth_query_decl::facade::application_program::{
    ApplicationActionDelivery, ApplicationSourceCompletion,
};
use worth_query_decl::facade::application_query::{
    ApplicationQueryOrderingDirection, ApplicationQueryRootPath,
};
use worth_query_host::facade::{
    application_installation::*, worth_query_application_program, WorthQueryApplicationHome,
};

pub struct CreatedAsset;
impl ApplicationActionDelivery<ShopFloorSchema, CreateAssetBinding> for CreatedAsset {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "hard_dx.create_asset.completion.v1";
}
pub struct UpdatedAsset;
impl ApplicationActionDelivery<ShopFloorSchema, UpdateAssetBinding> for UpdatedAsset {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "hard_dx.update_asset.completion.v1";
}
pub struct DeletedAsset;
impl ApplicationActionDelivery<ShopFloorSchema, DeleteAssetBinding> for DeletedAsset {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "hard_dx.delete_asset.completion.v1";
}

worth_query_application_program! {
    pub program Program for ShopFloorSchema {
        identity: "hard_dx.shop.program.v1",
        contribution: Contribution {
            identity: "hard_dx.shop.contribution.v1",
            members: [ShopFloor, PrincipalBinding],
        },
        managed_indexes: [
            TitlePrefixIndex,
            FocusIntervalIndex,
            EquipmentBox2DIndex,
            RouteBox3DIndex,
            FloatBox2DIndex,
        ],
        feature: ShopFeature => ShopDefinition / ShopRuntime {
            identity: "hard_dx.shop.feature.v1",
            query(AssetSourceBinding) {
                definition: AssetSourceQuery::definition()
                    .root_path(
                        ApplicationQueryRootPath::from(Floor::reference())
                            .forward(ContainsAsset::reference()),
                    )
                    .order_by(AssetRow::key(), ApplicationQueryOrderingDirection::Ascending);
            };
            query(FindByTitlePrefixBinding) {
                definition: FindByTitlePrefixQuery::definition()
                    .order_by(AssetRow::key(), ApplicationQueryOrderingDirection::Ascending);
            };
            query(FindByFocusWindowBinding) {
                definition: FindByFocusWindowQuery::definition()
                    .order_by(AssetRow::key(), ApplicationQueryOrderingDirection::Ascending);
            };
            query(FindEquipmentInBox2DBinding) {
                definition: FindEquipmentInBox2DQuery::definition()
                    .order_by(AssetRow::key(), ApplicationQueryOrderingDirection::Ascending);
            };
            query(FindRoutesInBox3DBinding) {
                definition: FindRoutesInBox3DQuery::definition()
                    .order_by(AssetRow::key(), ApplicationQueryOrderingDirection::Ascending);
            };
            query(FindInFloatBox2DBinding) {
                definition: FindInFloatBox2DQuery::definition()
                    .order_by(AssetRow::key(), ApplicationQueryOrderingDirection::Ascending);
            };
            action create(CreateAssetBinding, CreatedAsset, CreateAssetHandler) {
                operation: CreateAssetOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish();
            };
            action update(UpdateAssetBinding, UpdatedAsset, UpdateAssetHandler) {
                operation: UpdateAssetOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish();
            };
            action delete(DeleteAssetBinding, DeletedAsset, DeleteAssetHandler) {
                operation: DeleteAssetOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish();
            };
        }
    }
}

pub(super) type Host = WorthQueryProgramApplicationRuntime<ShopFloorSchema, Program>;
pub(super) type Home = WorthQueryApplicationHome<ShopFloorSchema, Program>;
