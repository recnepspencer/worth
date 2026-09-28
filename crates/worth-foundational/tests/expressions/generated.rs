//! Seeded property generation over the parser and codec. Every generated
//! input denies or round-trips exactly, and source and decoded drafts agree
//! on admission; failures reproduce from the fixed seeds.

use worth_foundational::expression_api::{expressions, ExpressionProfile};

use super::{draft, empty_catalog, schema};

/// A deterministic xorshift stream, so every failure reproduces from its seed.
struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound.max(1) as u64) as usize
    }
}

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
                let _ = decoded.admit(&schema, &catalog, ExpressionProfile::interactive());
            }
        }
    }
    assert!(
        decoded_count > 100,
        "mutations reach admission: {decoded_count}"
    );
}

const ATOMS: [&str; 18] = [
    "width",
    "depth",
    "count",
    "ready",
    "label",
    "frame.thickness",
    "frame.material",
    "Material::Steel",
    "1",
    "2.5",
    "\"s\"",
    "clear_width",
    "quantity(3.0, mm)",
    "bus",
    "members",
    "v",
    "m.thickness",
    "none<Float64>",
];
const BINARY: [&str; 12] = [
    "+", "-", "*", "/", "%", "<", "<=", "==", "!=", "&&", "||", "??",
];
const CALLS: [&str; 7] = [
    "abs",
    "sqrt",
    "length",
    "some",
    "is_some",
    "unwrap",
    "exact_cast<Int32>",
];
const CORRUPTIONS: [&str; 8] = ["(", ")", ",", "?", ":", ";", "\\", "\u{e9}"];

/// A random expression that is usually well formed and often well typed.
fn generate(random: &mut Seeded, depth: u32) -> String {
    if depth == 0 || random.below(3) == 0 {
        return ATOMS[random.below(ATOMS.len())].to_string();
    }
    let a = generate(random, depth - 1);
    let b = generate(random, depth - 1);
    let c = generate(random, depth - 1);
    match random.below(7) {
        0 => format!("({a} {} {b})", BINARY[random.below(BINARY.len())]),
        1 => format!("{}{a}", ["!", "-"][random.below(2)]),
        2 => format!("({a} ? {b} : {c})"),
        3 => format!("(let v = {a}; {b})"),
        4 => format!(
            "members.{}(m, {a})",
            ["map", "filter", "all", "any"][random.below(4)]
        ),
        5 => format!("[{a}, {b}]"),
        _ => format!("{}({a})", CALLS[random.below(CALLS.len())]),
    }
}

#[test]
fn generated_sources_deny_or_admit_consistently() {
    let schema = schema();
    let catalog = empty_catalog(&schema);
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
        let parsed = draft.admit(&schema, &catalog, ExpressionProfile::interactive());
        let rebuilt = decoded.admit(&schema, &catalog, ExpressionProfile::interactive());
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
