use crate::expressions::ordinary_context::{
    denials::{area, catalog},
    denied_family, schema,
};
use worth_foundational::expression_api::{expressions, Dynamic, ExpressionBuilder, Term};

pub(super) fn nongeneric_callees_deny_type_arguments() {
    // Source spells type arguments only on generic intrinsics; builders and
    // decoded drafts can attach them to any call.
    assert_eq!(
        expressions()
            .parse("geometry::area<Int8>(width, depth)")
            .unwrap_err()
            .family(),
        worth_foundational::expression_api::ExpressionDenialFamily::Syntax
    );
    let int8 = worth_foundational::expression_api::ExpressionTypeArgument::Type(
        worth_foundational::expression_api::ExpressionType::Integer(
            worth_foundational::expression_api::IntegerType::Int8,
        ),
    );
    let schema = schema();
    let catalog = catalog(&schema, [area()]);
    let calls: [(&str, fn(&mut ExpressionBuilder) -> Vec<Term<Dynamic>>); 6] = [
        ("geometry::area", |b| vec![b.name("width"), b.name("depth")]),
        ("int32", |b| vec![b.int(5).dynamic()]),
        ("float64", |b| vec![b.float(1.0).dynamic()]),
        ("decimal", |b| vec![b.text("1.5").dynamic()]),
        ("sqrt", |b| vec![b.name("width")]),
        ("bit_not", |b| vec![b.name("bus")]),
    ];
    for (function, arguments) in calls {
        let mut builder = worth_foundational::expression_api::ExpressionBuilder::new();
        let arguments = arguments(&mut builder);
        let call = builder.generic_call(function, std::slice::from_ref(&int8), &arguments);
        let denial = builder
            .finish(call)
            .and_then(|draft| {
                draft.admit(
                    &schema,
                    &catalog,
                    worth_foundational::expression_api::ExpressionProfile::interactive(),
                )
            })
            .unwrap_err();
        assert_eq!(
            denial.family(),
            worth_foundational::expression_api::ExpressionDenialFamily::UnsupportedFeature,
            "{function}"
        );
    }
    // Every reserved constructor but `bits<N>` denies type arguments.
    let constructors = [
        "int8",
        "int16",
        "int32",
        "int64",
        "uint8",
        "uint16",
        "uint32",
        "uint64",
        "float32",
        "float64",
        "decimal",
        "bytes",
        "logic4",
        "quantity",
        "magnitude",
    ];
    for function in constructors {
        let mut builder = worth_foundational::expression_api::ExpressionBuilder::new();
        let argument = builder.int(5).dynamic();
        let call = builder.generic_call(function, std::slice::from_ref(&int8), &[argument]);
        let denial = builder
            .finish(call)
            .and_then(|draft| {
                draft.admit(
                    &schema,
                    &catalog,
                    worth_foundational::expression_api::ExpressionProfile::interactive(),
                )
            })
            .unwrap_err();
        assert_eq!(
            denial.family(),
            worth_foundational::expression_api::ExpressionDenialFamily::UnsupportedFeature,
            "{function}"
        );
    }
    assert_eq!(
        denied_family("INT8(5)"),
        worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding
    );
    assert_eq!(
        denied_family("Int8(5)"),
        worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding
    );
}
