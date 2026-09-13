use worth_query::facade::consumer_kit::WorthQuerySupportPinReport;

pub fn certify_product_projection_support_contract() -> Result<WorthQuerySupportPinReport, String> {
    let source = crate::product_projection::shared_source_state();
    let runtime = crate::product_projection::projection_runtime_builder(source, Default::default())
        .map_err(|error| format!("{error:?}"))?
        .build()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .workspace("worth-ui-product-support-certification")
        .map_err(|error| error.to_string())?;

    crate::product_projection::evaluate_product_projection_support(&workspace)
        .map_err(|error| error.to_string())
}
