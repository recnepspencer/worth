use super::WorthSignalJsError;
use worth_signal::facade::{NodeId, SignalError};

#[test]
fn missing_bounded_inputs_keep_the_invalid_input_wire_code() {
    let node = NodeId::new(1, 0);
    for error in [
        SignalError::MissingBoundedInputsAtEpochAdmission { node },
        SignalError::MissingBoundedInputsAtEvaluation { node },
    ] {
        let message = error.to_string();
        let wire = serde_json::to_value(WorthSignalJsError::from(error)).unwrap();
        assert_eq!(
            wire["code"],
            serde_json::Value::String("invalidInput".to_owned()),
            "a caller's missing bounded inputs retain the invalidInput wire code: {message}"
        );
    }
}
