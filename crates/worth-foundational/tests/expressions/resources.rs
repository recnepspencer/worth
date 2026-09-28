//! Resource ceilings: every phase denies before unbounded work, and a draft
//! read under wide ceilings is re-checked against the admitting profile.

use worth_foundational::expression_api::{
    expressions, ExpressionBuilder, ExpressionDenial, ExpressionDenialDetail,
    ExpressionFunctionCatalog, ExpressionProfile, ExpressionResource, ExpressionSchema,
    ExpressionType,
};

use super::{draft, empty_catalog, function, schema};

fn exceeded(denial: &ExpressionDenial) -> ExpressionResource {
    match denial.detail() {
        ExpressionDenialDetail::ResourceExceeded { resource, .. } => *resource,
        other => panic!("expected a resource denial, found {other:?}"),
    }
}

fn narrowed(resource: ExpressionResource, limit: u64) -> ExpressionProfile {
    ExpressionProfile::interactive()
        .narrowed(resource, limit)
        .expect("narrowing lowers a ceiling")
}

#[test]
fn profiles_only_narrow() {
    let interactive = ExpressionProfile::interactive();
    for (resource, limit) in interactive.limits() {
        assert!(
            limit <= ExpressionProfile::engineering().limit(resource),
            "{resource:?}"
        );
        assert!(interactive.narrowed(resource, limit + 1).is_err());
        assert!(interactive.narrowed(resource, 0).is_err());
    }
}

#[test]
fn parsing_is_bounded_by_the_reading_profile() {
    let reading = |profile| expressions().reading_within(profile);
    let long = "width + ".repeat(20) + "width";
    let denial = reading(narrowed(ExpressionResource::SourceBytes, 16))
        .parse(&long)
        .unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::SourceBytes);
    let denial = reading(narrowed(ExpressionResource::SyntaxNodes, 8))
        .parse(&long)
        .unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::SyntaxNodes);

    let deep = "(".repeat(200) + "width" + &")".repeat(200);
    let denial = expressions().parse(&deep).unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::SyntaxDepth);
    let nested = "-".repeat(10_000) + "1.0";
    assert_eq!(
        exceeded(&expressions().parse(&nested).unwrap_err()),
        ExpressionResource::SyntaxDepth
    );
}

#[test]
fn admission_rechecks_drafts_read_under_wider_ceilings() {
    let schema = schema();
    let catalog = empty_catalog(&schema);
    let long = "width + ".repeat(20) + "width";
    let draft = draft(&long);
    let denial = draft
        .admit(
            &schema,
            &catalog,
            narrowed(ExpressionResource::SourceBytes, 16),
        )
        .unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::SourceBytes);
    let denial = draft
        .admit(
            &schema,
            &catalog,
            narrowed(ExpressionResource::SyntaxNodes, 8),
        )
        .unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::SyntaxNodes);
    let decoded = expressions().decode(&draft.encode()).expect("decodes");
    let denial = decoded
        .admit(
            &schema,
            &catalog,
            narrowed(ExpressionResource::DecodedBytes, 16),
        )
        .unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::DecodedBytes);
    assert!(decoded
        .admit(&schema, &catalog, ExpressionProfile::interactive())
        .is_ok());
}

#[test]
fn decoding_checks_counts_against_input_before_allocating() {
    let mut bytes = b"WXDR".to_vec();
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&60_000_u32.to_le_bytes());
    bytes.extend_from_slice(&[0, 1]);
    let denial = expressions().decode(&bytes).unwrap_err();
    assert!(matches!(
        denial.detail(),
        ExpressionDenialDetail::InvalidValue(_)
    ));

    let huge = vec![0_u8; 2 * 1024 * 1024];
    let reading = expressions().reading_within(ExpressionProfile::interactive());
    assert_eq!(
        exceeded(&reading.decode(&huge).unwrap_err()),
        ExpressionResource::DecodedBytes
    );
}

#[test]
fn builder_expansion_is_bounded() {
    let mut builder = ExpressionBuilder::new();
    let mut term = builder.name("width").numeric();
    for _ in 0..40 {
        term = builder.add(term, term);
    }
    assert_eq!(
        exceeded(&builder.finish(term).unwrap_err()),
        ExpressionResource::SyntaxNodes
    );

    let mut other = ExpressionBuilder::new();
    let foreign = other.boolean(true);
    let mut builder = ExpressionBuilder::new();
    let local = builder.boolean(false);
    let mixed = builder.and(local, foreign);
    assert!(
        builder.finish(mixed).is_err(),
        "terms belong to one builder"
    );
    assert!(ExpressionBuilder::new().finish(foreign).is_err());
}

#[test]
fn function_expansion_and_call_depth_are_bounded() {
    let schema = schema();
    let float = ExpressionType::Float64;
    let mut builder = ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive());
    builder
        .install(function(
            "bomb::f0",
            &[("x", float.clone())],
            float.clone(),
            "x + x",
        ))
        .expect("installs");
    let mut denial = None;
    for level in 1..40 {
        let body = format!(
            "bomb::f{previous}(x) + bomb::f{previous}(x)",
            previous = level - 1
        );
        let name = format!("bomb::f{level}");
        if let Err(error) = builder.install(function(
            &name,
            &[("x", float.clone())],
            float.clone(),
            &body,
        )) {
            denial = Some(error);
            break;
        }
    }
    let denial = denial.expect("doubling expansion reaches the ceiling");
    assert_eq!(exceeded(&denial), ExpressionResource::ExpandedInstructions);

    let mut builder = ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive());
    builder
        .install(function(
            "chain::f0",
            &[("x", float.clone())],
            float.clone(),
            "x",
        ))
        .expect("installs");
    let mut denial = None;
    for level in 1..40 {
        let body = format!("chain::f{}(x)", level - 1);
        let name = format!("chain::f{level}");
        if let Err(error) = builder.install(function(
            &name,
            &[("x", float.clone())],
            float.clone(),
            &body,
        )) {
            denial = Some(error);
            break;
        }
    }
    assert_eq!(
        exceeded(&denial.expect("the chain reaches the ceiling")),
        ExpressionResource::CallDepth
    );
}

#[test]
fn catalog_entries_and_bit_widths_are_bounded() {
    let schema = schema();
    let profile = narrowed(ExpressionResource::CatalogEntries, 1);
    let mut builder = ExpressionFunctionCatalog::builder(&schema, profile);
    let float = ExpressionType::Float64;
    builder
        .install(function("limits::a", &[], float.clone(), "1.0"))
        .expect("the first entry fits");
    let denial = builder
        .install(function("limits::b", &[], float.clone(), "2.0"))
        .unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::CatalogEntries);
    assert_eq!(
        builder.build().functions().len(),
        1,
        "a denied install leaves the catalog unchanged"
    );

    let wide = schema.clone();
    let catalog = empty_catalog(&wide);
    let draft = draft("concat(concat(bus, bus), bus)");
    let denial = draft
        .admit(&wide, &catalog, narrowed(ExpressionResource::BitWidth, 16))
        .unwrap_err();
    assert_eq!(exceeded(&denial), ExpressionResource::BitWidth);
}

#[test]
fn operand_lookups_charge_logarithmic_probes() {
    let work = |operands: usize| {
        let mut builder = ExpressionSchema::builder();
        for index in 0..operands {
            builder = builder
                .operand(&format!("x{index:04}"), ExpressionType::Float64)
                .expect("fixture operands are valid");
        }
        let schema = builder.build();
        let source = format!("[{}x0000]", "x0000, ".repeat(99));
        draft(&source)
            .admit(
                &schema,
                &empty_catalog(&schema),
                ExpressionProfile::engineering(),
            )
            .expect("admits")
            .admission_work()
    };
    let (small, large) = (work(1), work(1024));
    assert!(large > small, "probes into a larger table cost more");
    assert!(
        large - small <= 100 * 11,
        "each probe is logarithmic: {small} vs {large}"
    );
}

#[test]
fn installed_overload_resolution_is_metered() {
    let schema = schema();
    let work = |overloads: u32| {
        let mut builder =
            ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::engineering());
        for width in 1..=overloads {
            let bus = ExpressionType::Bits(width);
            let declaration = function(
                "digital::same",
                &[("a", bus.clone()), ("b", bus)],
                ExpressionType::Bool,
                "true",
            );
            builder
                .install(declaration)
                .expect("overloads differ by width");
        }
        let catalog = builder.build();
        let source = format!(
            "[{}digital::same(bus, bus)]",
            "digital::same(bus, bus), ".repeat(9)
        );
        draft(&source)
            .admit(&schema, &catalog, ExpressionProfile::engineering())
            .expect("admits")
            .admission_work()
    };
    let (few, many) = (work(8), work(512));
    // Ten calls, each visiting 504 more candidates for the name and two
    // parameters twice.
    assert!(many - few >= 10 * 504 * 5, "{few} vs {many}");
}
