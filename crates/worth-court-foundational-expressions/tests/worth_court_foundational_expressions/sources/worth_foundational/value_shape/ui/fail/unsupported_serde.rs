use worth_foundational::facade::WorthValue;

#[derive(WorthValue)]
struct SkippedField {
    #[serde(skip)]
    secret: String,
}

#[derive(WorthValue)]
struct DefaultField {
    #[serde(default)]
    children: Vec<String>,
}

#[derive(WorthValue)]
struct FlattenedField {
    #[serde(flatten)]
    child: Child,
}

#[derive(WorthValue)]
struct Child {
    count: u32,
}

fn main() {}
