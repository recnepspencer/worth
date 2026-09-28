use std::sync::{Arc, Mutex};

use crate::facade::change_source::RelationalRuntimeHandle;
use crate::tests::support::runtime_with_test_schema;

#[test]
fn a_shared_handle_describes_itself_while_its_runtime_is_held() {
    let runtime = runtime_with_test_schema();
    let expected = runtime.runtime_instance_id();
    let handle = RelationalRuntimeHandle::shared(Arc::new(Mutex::new(runtime)));

    let (id, debug) =
        handle.with_runtime(|_| (handle.runtime_instance_id(), format!("{handle:?}")));

    assert_eq!(id, expected);
    assert!(debug.contains(&format!("runtime_instance_id: {expected}")));
    assert!(debug.contains("shared"));
}
