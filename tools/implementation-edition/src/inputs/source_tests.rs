use super::content_digest;
use std::path::Path;

fn digest(source: &str) -> String {
    content_digest(Path::new("lib.rs"), source.as_bytes(), false).unwrap()
}

#[test]
fn enabled_test_inputs_are_semantic_while_inactive_tests_are_excluded() {
    let before = "#[cfg(test)] const VALUE:u8=17;";
    let after = "#[cfg(test)] const VALUE:u8=23;";
    assert_eq!(digest(before), digest(after));
    assert_ne!(
        content_digest(Path::new("lib.rs"), before.as_bytes(), true).unwrap(),
        content_digest(Path::new("lib.rs"), after.as_bytes(), true).unwrap()
    );
}

#[test]
fn native_imports_and_expansion_references_cannot_hide_behind_aliases() {
    for source in [
        "use core::arch::asm as emit; fn f(){ emit!(\"nop\"); }",
        "use std::arch::{global_asm as emit}; emit!(\"nop\");",
        "use core::arch::naked_asm as emit;",
        "macro_rules! import {($name:ident)=>{use core::arch::asm as $name;}} import!(emit);",
        "macro_rules! import {($path:path)=>{use $path as emit;}} import!(core::arch::asm);",
        "unsafe extern \"C\" { fn foreign(); }",
        "macro_rules! foreign {()=>{unsafe extern \"C\" {fn f();}}}",
    ] {
        let error = content_digest(Path::new("lib.rs"), source.as_bytes(), false).unwrap_err();
        assert!(error.contains("native source inputs"), "{source}: {error}");
    }
    digest("#[cfg(test)] use core::arch::asm as emit; pub const VALUE:u8=3;");
}

#[test]
fn rust_defined_abi_functions_do_not_introduce_foreign_inputs() {
    digest("extern \"C\" fn value()->u8{17} type Callback=extern \"C\" fn()->u8;");
    digest("macro_rules! define {()=>{extern \"C\" fn value()->u8{17}}} define!();");
    digest("fn value()->u8 { let asm=17; asm }");
}

#[test]
fn raw_native_names_are_classified_in_imports_direct_paths_and_expansions() {
    for source in [
        "use core::arch::r#asm as emit; fn f(){emit!(\"nop\");}",
        "use std::arch::{r#global_asm as emit}; emit!(\"nop\");",
        "use core::arch::r#naked_asm as emit;",
        "fn f(){core::arch::r#asm!(\"nop\");}",
        "r#global_asm!(\"nop\");",
        "macro_rules! import {($name:ident)=>{use core::arch::r#asm as $name;}} import!(emit);",
        "macro_rules! import {($path:path)=>{use $path as emit;}} import!(core::arch::r#asm);",
    ] {
        let error = content_digest(Path::new("lib.rs"), source.as_bytes(), false).unwrap_err();
        assert!(error.contains("native source inputs"), "{source}: {error}");
    }
    digest("extern crate r#core; fn value()->u8 { let r#asm=17; r#asm }");
}

#[test]
fn excluded_test_spans_keep_production_line_and_column_meaning() {
    let before = "#[cfg(test)] mod tests {}\npub const VALUE: u32 = line!();\n";
    let after = "#[cfg(test)] mod tests {\n}\npub const VALUE: u32 = line!();\n";
    assert_ne!(digest(before), digest(after));
    let same_positions = "#[cfg(test)] mod renamed_tests {}\npub const VALUE: u32 = line!();\n";
    assert_eq!(digest(before), digest(same_positions));
    let same_line = "#[cfg(test)] mod tests {} pub const VALUE: u32 = column!();\n";
    let shifted = "#[cfg(test)] mod longer_tests {} pub const VALUE: u32 = column!();\n";
    assert_ne!(digest(same_line), digest(shifted));
}

#[test]
fn trivia_is_normalized_without_rewriting_literals_or_doc_attributes() {
    assert_eq!(
        digest("pub const VALUE: u8=3;\n"),
        digest("// descriptive\r\npub const VALUE: u8 = 3;\r\n")
    );
    assert_ne!(
        digest("pub const VALUE: &str=\"a\\\\b\";"),
        digest("pub const VALUE: &str=\"a//b\";")
    );
    assert_ne!(
        digest("#[doc=\"first\"] pub const VALUE:u8=3;"),
        digest("#[doc=\"second\"] pub const VALUE:u8=3;")
    );
}
