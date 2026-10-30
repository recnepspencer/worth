use super::*;

#[test]
fn four_valued_values_have_no_implied_equality() {
    let schema = worth_foundational::expression_api::ExpressionSchema::builder()
        .record(
            "Sig",
            1,
            [(
                "line",
                worth_foundational::expression_api::ExpressionType::Logic4(1),
            )],
        )
        .and_then(|builder| {
            builder.operand(
                "enable",
                worth_foundational::expression_api::ExpressionType::Logic4(1),
            )
        })
        .and_then(|builder| {
            builder.operand(
                "sig",
                worth_foundational::expression_api::ExpressionType::Record(
                    worth_foundational::expression_api::ExpressionTypeName::new("Sig", 1)?,
                ),
            )
        })
        .expect("fixture schema is valid")
        .build();
    let denied = |source: &str| {
        draft(source)
            .admit(
                &schema,
                &empty_catalog(&schema),
                worth_foundational::expression_api::ExpressionProfile::interactive(),
            )
            .expect_err(source)
    };
    for (source, function) in [
        ("enable == enable", "=="),
        ("enable != enable", "!="),
        ("some(enable) == some(enable)", "=="),
        ("[enable] != [enable]", "!="),
        ("sig == sig", "=="),
        ("contains([enable], enable)", "contains"),
    ] {
        match denied(source).detail() {
            worth_foundational::expression_api::ExpressionDenialDetail::FunctionContractMismatch {
                function: named, ..
            } => {
                assert_eq!(named, function, "{source}");
            }
            other => panic!("{source} denied with {other:?}"),
        }
    }
    let admitted = |source: &str| {
        draft(source).admit(
            &schema,
            &empty_catalog(&schema),
            worth_foundational::expression_api::ExpressionProfile::interactive(),
        )
    };
    assert!(admitted("case_equal(sig.line, enable)").is_ok());
    let result = admitted("logic_eq(sig.line, enable) == logic_eq(enable, enable)");
    assert_eq!(
        result.unwrap_err().family(),
        worth_foundational::expression_api::ExpressionDenialFamily::FunctionContractMismatch
    );
}
