// Seeded property generation over the parser and codec. Every generated
// input denies or round-trips exactly, and source and decoded drafts agree
// on admission; failures reproduce from the fixed seeds.
#[allow(unused_imports)]
use crate::expressions::reference_context::generated::*;
pub(crate) use {
    worth_foundational::expression_api::expressions,
    worth_foundational::expression_api::ExpressionFunctionCatalog,
    worth_foundational::expression_api::ExpressionProfile,
    worth_foundational::expression_api::ExpressionSchema,
    worth_foundational::expression_api::ExpressionType,
};

pub(crate) use {
    crate::expressions::ordinary_context::draft,
    crate::expressions::ordinary_context::empty_catalog,
    crate::expressions::ordinary_context::function, crate::expressions::ordinary_context::schema,
};

#[test]
fn mutated_encodings_deny_or_round_trip_exactly() {
    let schema = schema();
    let catalog = empty_catalog(&schema);
    let sources = [
        "width - 2.0 * depth > 0.5 || frame.material == Material::Steel",
        "members.filter(m, m.thickness > quantity(1.0, mm)).map(m, m.thickness)",
        r#"let x = label ?? "none"; x == "a" ? [1.0, 2.0] : none<List<Float64>> ?? []"#,
        r#"slice<1, 3>(concat(bus, bits<8>("00001111")))"#,
    ];
    let mut random = Seeded(0x5EED_C0DE_C0DE_0001);
    let mut decoded_count = 0;
    for source in sources {
        let encoded = draft(source).encode();
        for round in 0..2_000 {
            let mut bytes = encoded.clone();
            match random.below(3) {
                0 => {
                    let at = random.below(bytes.len());
                    bytes[at] = random.next() as u8;
                }
                1 => bytes.truncate(random.below(bytes.len())),
                _ => {
                    let at = random.below(bytes.len() + 1);
                    bytes.insert(at, random.next() as u8);
                }
            }
            if let Ok(decoded) = expressions().decode(&bytes) {
                decoded_count += 1;
                assert_eq!(decoded.encode(), bytes, "{source}, round {round}");
                // Admission of a mutated draft is deterministic either way.
                let first = decoded.admit(
                    &schema,
                    &catalog,
                    worth_foundational::expression_api::ExpressionProfile::interactive(),
                );
                let again = decoded.admit(
                    &schema,
                    &catalog,
                    worth_foundational::expression_api::ExpressionProfile::interactive(),
                );
                match (first, again) {
                    (Ok(first), Ok(again)) => assert_eq!(first.identity(), again.identity()),
                    (Err(first), Err(again)) => assert_eq!(first.detail(), again.detail()),
                    _ => panic!("{source}, round {round}: admission is not deterministic"),
                }
            }
        }
    }
    assert!(
        decoded_count > 100,
        "mutations reach admission: {decoded_count}"
    );
}

#[test]
fn generated_sources_deny_or_admit_consistently() {
    let schema = schema();
    let catalog = catalog(&schema);
    let mut random = Seeded(0x5EED_70CE_0000_0002);
    let (mut parsed_count, mut admitted_count) = (0, 0);
    for round in 0..10_000 {
        let mut source = generate(&mut random, 4);
        if random.below(4) == 0 {
            let at = random.below(source.len());
            if source.is_char_boundary(at) {
                source.insert_str(at, CORRUPTIONS[random.below(CORRUPTIONS.len())]);
            }
        }
        let Ok(draft) = expressions().parse(&source) else {
            continue;
        };
        parsed_count += 1;
        let decoded = expressions()
            .decode(&draft.encode())
            .expect("encodings decode");
        assert_eq!(decoded.encode(), draft.encode(), "round {round}: {source}");
        let parsed = draft.admit(
            &schema,
            &catalog,
            worth_foundational::expression_api::ExpressionProfile::interactive(),
        );
        let rebuilt = decoded.admit(
            &schema,
            &catalog,
            worth_foundational::expression_api::ExpressionProfile::interactive(),
        );
        match (parsed, rebuilt) {
            (Ok(parsed), Ok(rebuilt)) => {
                admitted_count += 1;
                assert_eq!(
                    parsed.identity(),
                    rebuilt.identity(),
                    "round {round}: {source}"
                );
            }
            (Err(parsed), Err(rebuilt)) => assert_eq!(parsed.detail(), rebuilt.detail()),
            _ => panic!("round {round}: source and decoded drafts disagree: {source}"),
        }
    }
    assert!(
        parsed_count > 5_000 && admitted_count > 500,
        "{parsed_count} parsed, {admitted_count} admitted"
    );
}
