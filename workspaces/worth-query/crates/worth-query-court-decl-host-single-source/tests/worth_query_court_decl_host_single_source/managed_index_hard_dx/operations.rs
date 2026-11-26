//! Create, update, and delete mutations for the Hard DX Shop Floor + Daymark courts.
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_mutation_binding,
    worth_query_operation, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, WorthValue)]
pub struct AssetSpec {
    pub key: u64,
    pub title: String,
    pub start_min: i64,
    pub end_min: i64,
    pub min: [i64; 3],
    pub max: [i64; 3],
    pub fmin_bits: [u64; 2],
    pub fmax_bits: [u64; 2],
}

impl AssetSpec {
    pub fn i64_only(
        key: u64,
        title: impl Into<String>,
        interval: (i64, i64),
        min: [i64; 3],
        max: [i64; 3],
    ) -> Self {
        Self {
            key,
            title: title.into(),
            start_min: interval.0,
            end_min: interval.1,
            min,
            max,
            fmin_bits: [
                CanonicalF64::from_f64(min[0] as f64).0,
                CanonicalF64::from_f64(min[1] as f64).0,
            ],
            fmax_bits: [
                CanonicalF64::from_f64(max[0] as f64).0,
                CanonicalF64::from_f64(max[1] as f64).0,
            ],
        }
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct CreateAsset {
    pub floor: String,
    pub spec: AssetSpec,
}
worth_query_structured_value_binding!(pub CreateAssetInput for CreateAsset {
    identity: "hard_dx.shop.create_input.v1",
    capture_input: true
});
worth_query_operation!(pub CreateAssetOp for ShopFloorSchema, input CreateAssetInput;
    access {
        read_entities: [Floor];
        read_fields: [FloorId::at("FloorId")];
        writes: [
            AssetKey::at("AssetKey"),
            Title::at("Title"),
            StartMin::at("StartMin"),
            EndMin::at("EndMin"),
            MinX::at("MinX"),
            MinY::at("MinY"),
            MinZ::at("MinZ"),
            MaxX::at("MaxX"),
            MaxY::at("MaxY"),
            MaxZ::at("MaxZ"),
            FminX::at("FminX"),
            FminY::at("FminY"),
            FmaxX::at("FmaxX"),
            FmaxY::at("FmaxY")
        ];
        creates: [Asset];
        links: [ContainsAsset];
    }
);
fn create_scope(input: &CreateAsset) -> String {
    input.floor.clone()
}
worth_query_mutation_binding!(pub CreateAssetBinding for CreateAsset, schema ShopFloorSchema, context ShopContext,
    identity "hard_dx.shop.create_binding.v1", input CreateAssetInput,
    operation CreateAssetOp, result BodyResultBinding,
    decision WorthQueryInvariantMutationTarget<ShopFloorSchema, Floor>,
    handler identity "hard_dx.shop.create_handler.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope FloorId, field FloorId::at("FloorId"), value create_scope,
);
pub struct CreateAssetHandler;
impl OperationHandler<ShopFloorSchema, CreateAssetBinding> for CreateAssetHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[])
    }
    fn decide(
        &self,
        input: &CreateAsset,
        reader: &mut DecisionReader<'_, '_, '_, ShopFloorSchema, CreateAssetBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<ShopFloorSchema, Floor>,
        WorthQueryOperationBodyDenial,
    > {
        let result = reader
            .resolve_entity(FloorId::at("FloorId"), input.floor.clone())
            .and_then(|fl| {
                reader.entity(&fl, Floor::reference())?;
                reader.mutation_target(&fl)
            });
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &CreateAsset,
        fl_target: WorthQueryInvariantMutationTarget<ShopFloorSchema, Floor>,
        writer: &mut CandidateWriter<'_, ShopFloorSchema, CreateAssetBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let result = (|| {
            let fl = writer.projected_entity(&fl_target)?;
            let key =
                WorthQueryApplicationEntityKey::new(format!("asset-{}", input.spec.key)).unwrap();
            let asset = writer.create_entity_in_context(&fl, Asset::reference(), key)?;
            let s = &input.spec;
            writer.initialize_field(&asset, AssetKey::at("AssetKey"), s.key)?;
            writer.initialize_field(&asset, Title::at("Title"), s.title.clone())?;
            writer.initialize_field(&asset, StartMin::at("StartMin"), s.start_min)?;
            writer.initialize_field(&asset, EndMin::at("EndMin"), s.end_min)?;
            writer.initialize_field(&asset, MinX::at("MinX"), s.min[0])?;
            writer.initialize_field(&asset, MinY::at("MinY"), s.min[1])?;
            writer.initialize_field(&asset, MinZ::at("MinZ"), s.min[2])?;
            writer.initialize_field(&asset, MaxX::at("MaxX"), s.max[0])?;
            writer.initialize_field(&asset, MaxY::at("MaxY"), s.max[1])?;
            writer.initialize_field(&asset, MaxZ::at("MaxZ"), s.max[2])?;
            writer.initialize_field(&asset, FminX::at("FminX"), CanonicalF64(s.fmin_bits[0]))?;
            writer.initialize_field(&asset, FminY::at("FminY"), CanonicalF64(s.fmin_bits[1]))?;
            writer.initialize_field(&asset, FmaxX::at("FmaxX"), CanonicalF64(s.fmax_bits[0]))?;
            writer.initialize_field(&asset, FmaxY::at("FmaxY"), CanonicalF64(s.fmax_bits[1]))?;
            writer.link(
                ContainsAsset::reference(),
                format!("contains-{}", input.spec.key),
                &fl,
                &asset,
            )
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct UpdateAsset {
    pub key: u64,
    pub spec: AssetSpec,
}
worth_query_structured_value_binding!(pub UpdateAssetInput for UpdateAsset {
    identity: "hard_dx.shop.update_input.v1",
    capture_input: true
});
worth_query_operation!(pub UpdateAssetOp for ShopFloorSchema, input UpdateAssetInput;
    access {
        read_entities: [Asset];
        read_fields: [
            AssetKey::at("AssetKey"),
            Title::at("Title"),
            StartMin::at("StartMin"),
            EndMin::at("EndMin"),
            MinX::at("MinX"),
            MinY::at("MinY"),
            MinZ::at("MinZ"),
            MaxX::at("MaxX"),
            MaxY::at("MaxY"),
            MaxZ::at("MaxZ"),
            FminX::at("FminX"),
            FminY::at("FminY"),
            FmaxX::at("FmaxX"),
            FmaxY::at("FmaxY")
        ];
        writes: [
            Title::at("Title"),
            StartMin::at("StartMin"),
            EndMin::at("EndMin"),
            MinX::at("MinX"),
            MinY::at("MinY"),
            MinZ::at("MinZ"),
            MaxX::at("MaxX"),
            MaxY::at("MaxY"),
            MaxZ::at("MaxZ"),
            FminX::at("FminX"),
            FminY::at("FminY"),
            FmaxX::at("FmaxX"),
            FmaxY::at("FmaxY")
        ];
    }
);
fn update_scope(input: &UpdateAsset) -> u64 {
    input.key
}
worth_query_mutation_binding!(pub UpdateAssetBinding for UpdateAsset, schema ShopFloorSchema, context ShopContext,
    identity "hard_dx.shop.update_binding.v1", input UpdateAssetInput,
    operation UpdateAssetOp, result BodyResultBinding,
    decision WorthQueryInvariantMutationTarget<ShopFloorSchema, Asset>,
    handler identity "hard_dx.shop.update_handler.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope AssetKey, field AssetKey::at("AssetKey"), value update_scope,
);
pub struct UpdateAssetHandler;

thread_local! {
    static UPDATE_RACE: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        const { std::cell::RefCell::new(None) };
}

/// Runs `competitor` inside the next update decided on this thread, after its
/// target's pre-images were observed and before its candidate commits.
pub fn race_next_update_on_this_thread(competitor: impl FnOnce() + 'static) {
    UPDATE_RACE.with(|race| *race.borrow_mut() = Some(Box::new(competitor)));
}

fn run_update_race_hook() {
    if let Some(competitor) = UPDATE_RACE.with(|race| race.borrow_mut().take()) {
        competitor();
    }
}
impl OperationHandler<ShopFloorSchema, UpdateAssetBinding> for UpdateAssetHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[])
    }
    fn decide(
        &self,
        input: &UpdateAsset,
        reader: &mut DecisionReader<'_, '_, '_, ShopFloorSchema, UpdateAssetBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<ShopFloorSchema, Asset>,
        WorthQueryOperationBodyDenial,
    > {
        // No discarded reads: `write_target` observes the pre-image of every
        // field this operation declares both readable and writable.
        let result = reader
            .resolve_entity(AssetKey::at("AssetKey"), input.key)
            .and_then(|a| {
                reader.entity(&a, Asset::reference())?;
                reader.write_target(&a)
            });
        run_update_race_hook();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &UpdateAsset,
        target: WorthQueryInvariantMutationTarget<ShopFloorSchema, Asset>,
        writer: &mut CandidateWriter<'_, ShopFloorSchema, UpdateAssetBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let result = (|| {
            let asset = writer.projected_entity(&target)?;
            let s = &input.spec;
            writer.write_field(&asset, Title::at("Title"), s.title.clone())?;
            writer.write_field(&asset, StartMin::at("StartMin"), s.start_min)?;
            writer.write_field(&asset, EndMin::at("EndMin"), s.end_min)?;
            writer.write_field(&asset, MinX::at("MinX"), s.min[0])?;
            writer.write_field(&asset, MinY::at("MinY"), s.min[1])?;
            writer.write_field(&asset, MinZ::at("MinZ"), s.min[2])?;
            writer.write_field(&asset, MaxX::at("MaxX"), s.max[0])?;
            writer.write_field(&asset, MaxY::at("MaxY"), s.max[1])?;
            writer.write_field(&asset, MaxZ::at("MaxZ"), s.max[2])?;
            writer.write_field(&asset, FminX::at("FminX"), CanonicalF64(s.fmin_bits[0]))?;
            writer.write_field(&asset, FminY::at("FminY"), CanonicalF64(s.fmin_bits[1]))?;
            writer.write_field(&asset, FmaxX::at("FmaxX"), CanonicalF64(s.fmax_bits[0]))?;
            writer.write_field(&asset, FmaxY::at("FmaxY"), CanonicalF64(s.fmax_bits[1]))
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct DeleteAsset {
    pub floor: String,
    pub key: u64,
}
worth_query_structured_value_binding!(pub DeleteAssetInput for DeleteAsset {
    identity: "hard_dx.shop.delete_input.v1",
    capture_input: true
});
worth_query_operation!(pub DeleteAssetOp for ShopFloorSchema, input DeleteAssetInput;
    access {
        read_entities: [Floor, Asset];
        read_fields: [FloorId::at("FloorId"), AssetKey::at("AssetKey")];
        read_relations: [ContainsAsset];
        deletes: [Asset];
        unlinks: [ContainsAsset];
    }
);
fn delete_scope(input: &DeleteAsset) -> String {
    input.floor.clone()
}
pub struct DeleteDecision {
    floor: WorthQueryInvariantMutationTarget<ShopFloorSchema, Floor>,
    asset: WorthQueryInvariantMutationTarget<ShopFloorSchema, Asset>,
}
worth_query_mutation_binding!(pub DeleteAssetBinding for DeleteAsset, schema ShopFloorSchema, context ShopContext,
    identity "hard_dx.shop.delete_binding.v1", input DeleteAssetInput,
    operation DeleteAssetOp, result BodyResultBinding,
    decision DeleteDecision,
    handler identity "hard_dx.shop.delete_handler.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope FloorId, field FloorId::at("FloorId"), value delete_scope,
);
pub struct DeleteAssetHandler;
impl OperationHandler<ShopFloorSchema, DeleteAssetBinding> for DeleteAssetHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[])
    }
    fn decide(
        &self,
        input: &DeleteAsset,
        reader: &mut DecisionReader<'_, '_, '_, ShopFloorSchema, DeleteAssetBinding>,
    ) -> HandlerResult<DeleteDecision, WorthQueryOperationBodyDenial> {
        let result = (|| {
            let floor = reader.resolve_entity(FloorId::at("FloorId"), input.floor.clone())?;
            let asset = reader.resolve_entity(AssetKey::at("AssetKey"), input.key)?;
            reader.entity(&floor, Floor::reference())?;
            reader.entity(&asset, Asset::reference())?;
            reader
                .relations_from(ContainsAsset::reference(), &floor)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(DeleteDecision {
                floor: reader.mutation_target(&floor)?,
                asset: reader.mutation_target(&asset)?,
            })
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &DeleteAsset,
        decision: DeleteDecision,
        writer: &mut CandidateWriter<'_, ShopFloorSchema, DeleteAssetBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let result = (|| {
            let floor = writer.projected_entity(&decision.floor)?;
            let asset = writer.projected_entity(&decision.asset)?;
            writer.unlink(ContainsAsset::reference(), &floor, &asset)?;
            writer.delete_entity(Asset::reference(), &asset)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
