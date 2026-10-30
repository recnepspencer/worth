// Seeded property generation over the parser and codec. Every generated
// input denies or round-trips exactly, and source and decoded drafts agree
// on admission; failures reproduce from the fixed seeds.
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

/// One installed overload, so generated calls exercise installed resolution.
pub(crate) fn catalog(schema: &ExpressionSchema) -> ExpressionFunctionCatalog {
    let mut builder = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(
        schema,
        worth_foundational::expression_api::ExpressionProfile::interactive(),
    );
    let area = function(
        "geometry::area",
        &[
            (
                "w",
                worth_foundational::expression_api::ExpressionType::Float64,
            ),
            (
                "h",
                worth_foundational::expression_api::ExpressionType::Float64,
            ),
        ],
        worth_foundational::expression_api::ExpressionType::Float64,
        "w * h",
    );
    builder.install(area).expect("area installs");
    builder.build()
}

pub(crate) struct Seeded(pub(crate) u64);

impl Seeded {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub(crate) fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound.max(1) as u64) as usize
    }
}

pub(crate) const ATOMS: [&str; 21] = [
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
    "int8(5)",
    "bits<4>(\"1010\")",
    "decimal(\"1.25\")",
];
pub(crate) const BINARY: [&str; 12] = [
    "+", "-", "*", "/", "%", "<", "<=", "==", "!=", "&&", "||", "??",
];
pub(crate) const CALLS: [&str; 7] = [
    "abs",
    "sqrt",
    "length",
    "some",
    "is_some",
    "unwrap",
    "exact_cast<Int32>",
];
pub(crate) const CORRUPTIONS: [&str; 8] = ["(", ")", ",", "?", ":", ";", "\\", "\u{e9}"];

/// A random expression that is usually well formed and often well typed.
pub(crate) fn generate(random: &mut Seeded, depth: u32) -> String {
    if depth == 0 || random.below(3) == 0 {
        return ATOMS[random.below(ATOMS.len())].to_string();
    }
    let a = generate(random, depth - 1);
    let b = generate(random, depth - 1);
    let c = generate(random, depth - 1);
    match random.below(10) {
        0 => format!("({a} {} {b})", BINARY[random.below(BINARY.len())]),
        1 => format!("{}{a}", ["!", "-"][random.below(2)]),
        2 => format!("({a} ? {b} : {c})"),
        3 => format!("(let v = {a}; {b})"),
        4 => format!(
            "members.{}(m, {a})",
            ["map", "filter", "all", "any"][random.below(4)]
        ),
        5 => format!("[{a}, {b}]"),
        6 if random.below(2) == 0 => format!("Frame {{ thickness: {a}, material: {b} }}"),
        6 => format!("Frame {{ material: {b}, thickness: {a} }}"),
        7 => format!("{{\"k\": {a}, \"j\": {b}}}"),
        8 if random.below(4) == 0 => format!("geometry::area<Int8>({a}, {b})"),
        8 => format!("geometry::area({a}, {b})"),
        _ => format!("{}({a})", CALLS[random.below(CALLS.len())]),
    }
}
