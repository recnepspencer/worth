//! Query-free authored values, with deliberately nondefault nested contents.
use worth_foundational::facade::WorthValue;

#[derive(Clone, Debug, PartialEq, WorthValue)]
pub struct NestedValue {
    pub title: String,
    #[serde(rename = "selection")]
    pub selected: Selection,
    pub children: Vec<Child>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, WorthValue)]
pub enum Selection {
    Idle,
    #[serde(rename = "chosen")]
    Selected(Child),
}

#[derive(Clone, Debug, PartialEq, WorthValue)]
pub struct Child {
    pub tag: Tag,
    pub active: bool,
    pub rank: i32,
}

#[derive(Clone, Debug, PartialEq, WorthValue)]
pub struct Tag(pub String);

pub fn authored_value() -> NestedValue {
    NestedValue {
        title: "assembly β".to_owned(),
        selected: Selection::Selected(Child {
            tag: Tag("primary".to_owned()),
            active: true,
            rank: -7,
        }),
        children: vec![
            Child {
                tag: Tag("left".to_owned()),
                active: false,
                rank: 23,
            },
            Child {
                tag: Tag("right".to_owned()),
                active: true,
                rank: -11,
            },
        ],
        notes: Some("line\nbreak".to_owned()),
    }
}

pub const AUTHORED_WIRE: &str = r#"{"title":"assembly β","selection":{"chosen":{"tag":"primary","active":true,"rank":-7}},"children":[{"tag":"left","active":false,"rank":23},{"tag":"right","active":true,"rank":-11}],"notes":"line\nbreak"}"#;
