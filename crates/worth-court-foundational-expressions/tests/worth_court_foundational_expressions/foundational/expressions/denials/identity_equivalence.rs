use crate::expressions::ordinary_context::{
    admit,
    denials::{area, catalog},
    draft, function, schema,
};
use worth_foundational::expression_api::ExpressionFunctionCatalog;

pub(super) fn binder_and_catalog_changes_preserve_expected_identity() {
    let identity = |source: &str| admit(source).expect(source).identity().clone();
    assert_eq!(
        identity("let a = width; a + a"),
        identity("let b = width; b + b")
    );
    assert_ne!(identity("extend<16>(bus)"), identity("extend<17>(bus)"));

    let schema = schema();
    let perimeter = function(
        "geometry::perimeter",
        &[(
            "w",
            worth_foundational::expression_api::ExpressionType::Float64,
        )],
        worth_foundational::expression_api::ExpressionType::Float64,
        "w * 4.0",
    );
    let call = |catalog: &ExpressionFunctionCatalog| {
        draft("geometry::area(width, depth)")
            .admit(
                &schema,
                catalog,
                worth_foundational::expression_api::ExpressionProfile::interactive(),
            )
            .expect("the call admits")
            .identity()
            .clone()
    };
    let alone = call(&catalog(&schema, [area()]));
    assert_eq!(alone, call(&catalog(&schema, [area(), perimeter.clone()])));
    assert_eq!(alone, call(&catalog(&schema, [perimeter, area()])));

    let mut bumped = area();
    bumped.name = worth_foundational::expression_api::ExpressionTypeName::new("geometry::area", 2)
        .expect("valid name");
    assert_ne!(alone, call(&catalog(&schema, [bumped])));
}
