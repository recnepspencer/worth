use crate::expressions::ordinary_context::{
    denials::{catalog, exceeded, narrowed},
    draft, empty_catalog, function, schema,
};

pub(super) fn work_and_canonical_byte_limits_are_enforced() {
    let schema = schema();
    let catalog = empty_catalog(&schema);
    let long = format!("[{}width]", "width, ".repeat(200));
    let result = draft(&long).admit(
        &schema,
        &catalog,
        narrowed(
            worth_foundational::expression_api::ExpressionResource::AdmissionWork,
            500,
        ),
    );
    assert_eq!(
        exceeded(result),
        worth_foundational::expression_api::ExpressionResource::AdmissionWork
    );
    let text = format!(r#"label ?? "{}""#, "x".repeat(400));
    let result = draft(&text).admit(
        &schema,
        &catalog,
        narrowed(
            worth_foundational::expression_api::ExpressionResource::CanonicalBytes,
            256,
        ),
    );
    assert_eq!(
        exceeded(result),
        worth_foundational::expression_api::ExpressionResource::CanonicalBytes
    );
}

pub(super) fn record_and_callee_bus_widths_are_bounded() {
    let wide = worth_foundational::expression_api::ExpressionSchema::builder()
        .record(
            "Wide",
            1,
            [(
                "bus",
                worth_foundational::expression_api::ExpressionType::Bits(128),
            )],
        )
        .and_then(|builder| {
            builder.operand(
                "wide",
                worth_foundational::expression_api::ExpressionType::Record(
                    worth_foundational::expression_api::ExpressionTypeName::new("Wide", 1)?,
                ),
            )
        })
        .expect("fixture schema is valid")
        .build();
    let result = draft("wide").admit(
        &wide,
        &empty_catalog(&wide),
        narrowed(
            worth_foundational::expression_api::ExpressionResource::BitWidth,
            64,
        ),
    );
    assert_eq!(
        exceeded(result),
        worth_foundational::expression_api::ExpressionResource::BitWidth
    );

    let schema = schema();
    let hidden = function(
        "digital::wide",
        &[],
        worth_foundational::expression_api::ExpressionType::Bool,
        r#"slice<0, 1>(to_bits<128>(1)) == bits<1>("1")"#,
    );
    let catalog = catalog(&schema, [hidden]);
    let result = draft("digital::wide()").admit(
        &schema,
        &catalog,
        narrowed(
            worth_foundational::expression_api::ExpressionResource::BitWidth,
            64,
        ),
    );
    assert_eq!(
        exceeded(result),
        worth_foundational::expression_api::ExpressionResource::BitWidth
    );
}
