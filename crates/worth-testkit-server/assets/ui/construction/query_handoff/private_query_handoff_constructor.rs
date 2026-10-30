use worth_server::{WorthServerQueryHandoff, WorthServerQueryHandoffOperation, WorthServerQuerySupportPosture};

fn main() {
    let _ = WorthServerQueryHandoff {
        admission: loop {},
        operation: WorthServerQueryHandoffOperation::query_read("users.profile"),
        workspace: loop {},
        downstream_delivery_contract: loop {},
        support_posture: WorthServerQuerySupportPosture::QueryReadSupported {
            family_contract: loop {},
        },
        canonical_digest: "Worthd".to_string(),
    };
}
