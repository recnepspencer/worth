use super::*;

#[test]
fn writer_preserves_persistent_naming_field_order_and_final_newline() {
    let mut trace = Trace::default();
    let set = r#"{"__set__":[]}"#;
    let row = format!(
        r#"{{"action":"Observe","resolved":{set},"postLive":{set},"postRetired":{set},"postEventCount":0}}"#
    );
    trace.push_json(&row).unwrap();
    let mut bytes = Vec::new();
    trace.write(&mut bytes).unwrap();
    assert_eq!(bytes, include_bytes!("persistent_naming.ndjson"));
}

#[test]
fn malformed_or_multiple_rows_leave_the_trace_unchanged() {
    let mut trace = Trace::default();
    trace.push_json(r#"{"step":0}"#).unwrap();
    for bad in ["", "true", "{", "{}\n{}", "{\n}", " {}"] {
        assert_eq!(
            trace.push_json(bad).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
    assert_eq!(trace.as_bytes(), b"{\"step\":0}\n");
}
