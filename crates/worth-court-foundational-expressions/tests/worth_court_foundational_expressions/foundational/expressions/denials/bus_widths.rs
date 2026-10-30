use super::*;

#[test]
fn bus_widths_are_checked_through_records_and_callees() {
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
