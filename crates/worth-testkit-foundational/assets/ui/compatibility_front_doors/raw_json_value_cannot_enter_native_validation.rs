use worth_foundational::{aspects, ScalarAspectType};
use serde_json::json;

fn main() {
    let vocabulary = aspects().vocabulary();
    let contract = aspects()
        .contract()
        .for_key(vocabulary.key("retry.count").expect("valid aspect key"))
        .identified_by(vocabulary.identity(1))
        .at_revision(vocabulary.revision(1))
        .scalar(ScalarAspectType::Int64);

    let _ = aspects().validate().against(&contract).value(json!(3));
}
