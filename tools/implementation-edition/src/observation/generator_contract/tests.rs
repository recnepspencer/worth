use super::Program;
use std::{collections::BTreeMap, fs, path::PathBuf};

fn directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "edition-effects-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    fs::canonicalize(path).unwrap()
}

#[test]
fn unknown_effects_are_refused_in_helpers_and_inactive_branches() {
    for source in [
        "fn main(){if false {std::fs::remove_file(\"other\");}}",
        "fn helper(){std::net::TcpStream::connect(\"localhost\");} fn main(){}",
        "fn main(){std::thread::spawn(||{});}",
        "fn main(){let mut p=std::env::var(\"OUT_DIR\").unwrap();if false {p += \"/..\";}let out=std::path::PathBuf::from(p);std::fs::write(out.join(\"proof\"),\"x\").unwrap();}",
        "fn main(){if false {let x=1+2;}}",
        "fn main(){if false {let x=!0u8;}}",
        "fn main(){if false {let x=\"256\".parse::<u16>().ok();}}",
        "fn main(){let mut p=std::env::var(\"OUT_DIR\").unwrap();if false {p=\"other\";}let out=std::path::PathBuf::from(p);std::fs::write(out.join(\"proof\"),\"x\").unwrap();}",
        "use std::fs as r#files; fn main(){r#files::remove_file(\"other\");}",
        "fn main(){if false {std::process::Command::new(\"other\").arg(\"--version\").output();}}",
        "fn main(){let rustc=std::env::var_os(\"RUSTC\").unwrap();if false {std::process::Command::new(rustc).output();}}",
        "fn main(){let rustc=std::env::var_os(\"RUSTC\").unwrap();if false {std::process::Command::new(rustc).arg(\"--version\").arg(\"--version\").output();}}",
        "fn main(){let rustc=std::env::var_os(\"RUSTC\").unwrap();let mut command=std::process::Command::new(rustc);command.arg(\"--version\");command.arg(\"--version\").output();}",
        "fn main(){if false {std::fs::write(\"outside\",\"data\");}}",
        "fn main(){let rustc=std::env::var_os(\"RUSTC\").unwrap();match Some(\"other\"){Some(rustc)=>{std::process::Command::new(rustc).arg(\"--version\").output();},None=>return}}",
    ] {
        assert!(Program::parse(source).is_err(), "{source}");
    }
}

#[test]
fn numeric_width_follows_explicit_local_and_helper_return_types() {
    let root = directory();
    for (source, expected) in [
        ("fn main(){if \"256\".parse::<u8>().ok()==None {println!(\"cargo:warning=u8-overflow\");}}", "cargo:warning=u8-overflow\n"),
        ("fn main(){let value:u8=\"255\".parse().unwrap();println!(\"cargo:warning={value}\");}", "cargo:warning=255\n"),
        ("fn version()->Option<u32>{\"256\".parse().ok()}fn main(){if version()==Some(256){println!(\"cargo:warning=u32-value\");}}", "cargo:warning=u32-value\n"),
    ] {
        let effects = Program::parse(source).unwrap().evaluate_with_environment(&root, &root, &root, &BTreeMap::new()).unwrap();
        assert_eq!(effects.stdout, expected);
    }
    assert!(
        Program::parse("fn main(){let inferred=\"1\".parse().ok();}")
            .err()
            .unwrap()
            .contains("parse type is unresolved")
    );
    fs::remove_dir(root).unwrap();
}

#[test]
fn validated_reads_require_actually_emitted_coverage() {
    let root = directory();
    fs::write(root.join("seed.txt"), "17").unwrap();
    let program = Program::parse("fn main(){if false {println!(\"cargo::rerun-if-changed=seed.txt\");} let seed=std::fs::read_to_string(\"seed.txt\").unwrap();}").unwrap();
    let error = program
        .evaluate_with_environment(&root, &root, &root, &BTreeMap::new())
        .err()
        .unwrap();
    assert!(error.contains("emitted directives omit validated file read"));
    fs::remove_file(root.join("seed.txt")).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn selected_compiler_query_and_output_effects_are_not_arbitrary_commands() {
    let root = directory();
    let selected = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
    let other = root.join("other.exe");
    fs::write(&other, "not a compiler").unwrap();
    let environment = BTreeMap::from([("RUSTC".into(), other.as_os_str().to_owned())]);
    let program = Program::parse("use std::env; use std::process::Command; fn query(){let rustc=env::var_os(\"RUSTC\").unwrap();Command::new(rustc).arg(\"--version\").output().ok();} fn main(){query();}").unwrap();
    assert!(program
        .evaluate_with_environment(&root, &root, &selected, &environment)
        .err()
        .unwrap()
        .contains("differs from selected compiler"));
    let query = "use std::env; use std::process::Command; fn main(){let rustc=env::var_os(\"RUSTC\").unwrap();Command::new(rustc).arg(\"--sysroot\").output().ok();}";
    assert!(Program::parse(query)
        .err()
        .unwrap()
        .contains("selected rustc --version"));
    fs::remove_file(other).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn helper_control_flow_and_generated_bytes_follow_actual_values() {
    let root = directory();
    let environment = BTreeMap::from([
        ("OUT_DIR".into(), root.as_os_str().to_owned()),
        ("CARGO_PKG_VERSION_PATCH".into(), "229".into()),
    ]);
    let program = Program::parse("use std::{env,fs,path::PathBuf}; const PRIVATE:&str=\"pub mod __private$$ {}\"; fn version()->Option<u32>{let mut pieces=\"rustc 1.94.0\".split('.'); if pieces.next()!=Some(\"rustc 1\"){return None;} pieces.next()?.parse().ok()} fn main(){let patch=env::var(\"CARGO_PKG_VERSION_PATCH\").unwrap();let module=PRIVATE.replace(\"$$\",&patch);let minor=match version(){Some(v)=>v,None=>return};if minor>=77{println!(\"cargo:rustc-cfg=current\");}let out=PathBuf::from(env::var_os(\"OUT_DIR\").unwrap());fs::write(out.join(\"private.rs\"),module).unwrap();}").unwrap();
    let effects = program
        .evaluate_with_environment(&root, &root, &root, &environment)
        .unwrap();
    assert_eq!(
        effects.writes[&root.join("private.rs")],
        b"pub mod __private229 {}"
    );
    assert_eq!(effects.stdout, "cargo:rustc-cfg=current\n");
    assert!(!root.join("private.rs").exists());
    fs::remove_dir(root).unwrap();
}

#[test]
fn real_serde_generator_source_has_no_identity_based_effect_exemption() {
    let source = crate::package_content::tests::serde_generator();
    Program::parse(&source).unwrap();
    let unknown = source + "\nfn unseen(){std::fs::remove_file(\"arbitrary\");}\n";
    assert!(Program::parse(&unknown)
        .err()
        .unwrap()
        .contains("effect requires qualification"));
}

#[test]
fn real_serde_helper_query_and_emitted_coverage_are_enforced() {
    let source = crate::package_content::tests::serde_generator();
    let root = directory();
    fs::write(root.join("build.rs"), &source).unwrap();
    fs::write(root.join("seed.txt"), "17").unwrap();
    let compiler =
        fs::canonicalize(crate::compiler_context::compiler_path("rustc").unwrap()).unwrap();
    let environment = BTreeMap::from([
        ("OUT_DIR".into(), root.as_os_str().to_owned()),
        ("RUSTC".into(), compiler.as_os_str().to_owned()),
        ("CARGO_PKG_VERSION_PATCH".into(), "229".into()),
        ("TARGET".into(), "x86_64-pc-windows-msvc".into()),
    ]);
    let query = source.replace(".arg(\"--version\")", ".arg(\"--sysroot\")");
    assert_ne!(source, query);
    assert!(Program::parse(&query)
        .err()
        .unwrap()
        .contains("selected rustc --version"));
    let omission = source.replacen(
        "fn main() {",
        "fn main() { let seed=fs::read_to_string(\"seed.txt\").unwrap();",
        1,
    );
    assert!(Program::parse(&omission)
        .unwrap()
        .evaluate_with_environment(&root, &root, &compiler, &environment)
        .err()
        .unwrap()
        .contains("omit validated file read"));
    fs::remove_file(root.join("seed.txt")).unwrap();
    fs::remove_file(root.join("build.rs")).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn original_finite_generator_keeps_its_actual_read_and_generated_content() {
    let root = directory();
    fs::write(root.join("seed.txt"), "1\n").unwrap();
    let source = include_str!("../../../tests/fixtures/closure/neutral/build.rs");
    fs::write(root.join("build.rs"), source).unwrap();
    let environment = BTreeMap::from([("OUT_DIR".into(), root.as_os_str().to_owned())]);
    let effects = Program::parse(source)
        .unwrap()
        .evaluate_with_environment(&root, &root, &root, &environment)
        .unwrap();
    assert_eq!(effects.reads[&root.join("seed.txt")], b"1\n");
    assert_eq!(
        effects.writes[&root.join("generated.rs")],
        b"pub const GENERATED: u8 = 1;\n"
    );
    fs::remove_file(root.join("seed.txt")).unwrap();
    fs::remove_file(root.join("build.rs")).unwrap();
    fs::remove_dir(root).unwrap();
}
