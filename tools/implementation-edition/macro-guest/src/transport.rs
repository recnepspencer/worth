//! The only guest imports are compiler-object operations. No ambient WASI.
#[link(wasm_import_module = "worth-macro-v1")]
unsafe extern "C" {
    fn invoke(op: u32, a: u32, b: u32, c: u32) -> u32;
    fn new_text(pointer: *const u8, length: u32) -> u32;
    fn read_text(handle: u32, pointer: *mut u8, length: u32);
}
pub(crate) fn call(op: u32, a: u32, b: u32, c: u32) -> u32 {
    // The host validates operation, object kind and invocation-scoped handle.
    unsafe { invoke(op, a, b, c) }
}
pub(crate) fn text_handle(value: &str) -> u32 {
    unsafe { new_text(value.as_ptr(), value.len().try_into().unwrap()) }
}
pub(crate) fn text(handle: u32) -> String {
    let length = call(1, handle, 0, 0);
    let mut bytes = vec![0; length as usize];
    unsafe {
        read_text(handle, bytes.as_mut_ptr(), length);
    }
    String::from_utf8(bytes).unwrap()
}
