use std::sync::Arc;

pub(super) fn panic_value_message(panic_value: Box<dyn std::any::Any + Send>) -> Arc<str> {
    if let Some(message) = panic_value.downcast_ref::<&'static str>() {
        return Arc::from(*message);
    }
    if let Some(message) = panic_value.downcast_ref::<String>() {
        return Arc::from(message.as_str());
    }
    Arc::from("custom invariant panicked with a non-string value")
}
