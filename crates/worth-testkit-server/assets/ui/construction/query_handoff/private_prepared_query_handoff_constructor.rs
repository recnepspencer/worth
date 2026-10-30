use worth_server::WorthServerPreparedQueryHandoffIntent;

fn main() {
    let _ = WorthServerPreparedQueryHandoffIntent {
        kind: panic!("sealed"),
        operation_name: "users.profile".to_string(),
    };
}
