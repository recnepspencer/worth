use serde_json::json;
use worth_server::{WorthServerProductOperationSuccess, WorthServerProductResultContract};

fn main() {
    let contract = WorthServerProductResultContract::canonical_json(
        "product.connection.result.v1",
        1,
        1024,
    )
    .unwrap();
    let naked_json = json!({ "connection_id": "host-7" });
    let _ = WorthServerProductOperationSuccess::publish_json(
        "connection-result",
        &contract,
        &naked_json,
    );
}
