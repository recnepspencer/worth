//! Protocol metadata projection, not native authorization or business execution.
use std::sync::Arc;
use worth_server::{
    surfaces::CompatHttpSurface, WorthServer, WorthServerConfig, WorthServerOperationRegistration,
    WorthServerProductAdapterExecutionError, WorthServerProductApplicationAdapter,
    WorthServerProductApplicationAdapterRegistration, WorthServerProductOperationBasisKind,
    WorthServerProductOperationDeclaration, WorthServerProductOperationErrorMaps,
    WorthServerProductOperationSuccess, WorthServerProductOperationSupportSnapshot,
    WorthServerProductResultContract, WorthServerScheduledProductOperation,
};
struct MetadataOnly;
impl WorthServerProductApplicationAdapter for MetadataOnly {
    fn execute(
        &self,
        _: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        Err(WorthServerProductAdapterExecutionError::failed(
            "metadata_only",
            "this test never executes a product operation",
        ))
    }
}
#[test]
fn projectors_preserve_numeric_version_independently_of_schema_identity_spelling() {
    let declaration = WorthServerProductOperationDeclaration::product_read(
        "codegen.describe",
        "codegen.input.v1",
        WorthServerProductResultContract::canonical_json("codegen.result.v1", 37, 1024).unwrap(),
        WorthServerProductOperationBasisKind::DurableProductDerived,
        WorthServerProductOperationSupportSnapshot::production_admitted("metadata-only-fixture"),
    )
    .with_error_map(WorthServerProductOperationErrorMaps::passthrough());
    let server = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8090).into())
                .build()
                .unwrap(),
        )
        .register_operations(WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .register_product_adapter(
            WorthServerProductApplicationAdapterRegistration::new(
                "codegen",
                Arc::new(MetadataOnly),
            )
            .with_operation(declaration),
        )
        .build()
        .unwrap();
    let catalog = server.product_protocol_catalog().unwrap();
    let typescript = crate::render_typescript_product_protocol(&catalog);
    let python = crate::render_python_product_protocol(&catalog);
    assert!(typescript.contains("resultSchemaVersion: 37"));
    assert!(python.contains("result_schema_version=37"));
}
