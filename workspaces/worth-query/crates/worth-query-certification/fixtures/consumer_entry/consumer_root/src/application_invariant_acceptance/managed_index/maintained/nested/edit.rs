//! Serialized nested graph edit shared by the nested and event adapters.
#[derive(Clone, Debug, serde::Serialize)]
pub enum Edit {
    Directory {
        identity: String,
    },
    Item {
        directory: String,
        identity: String,
        name: String,
        root: bool,
        incarnation: u64,
    },
    Name {
        item: String,
        value: String,
    },
    Badge {
        item: String,
        value: Option<(bool, Option<bool>)>,
    },
    Enabled {
        item: String,
        value: bool,
    },
    Link {
        from: String,
        to: String,
        edition: u64,
    },
    Unlink {
        from: String,
        to: String,
    },
    Share {
        directory: String,
        item: String,
    },
    Retire {
        item: String,
    },
}
