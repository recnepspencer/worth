"""Real source-built Serde guest and trusted native live-token broker subset.

This is subsystem qualification, not ordinary observer macro admission. The
production compiler/dependency construction owner remains required.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import time
import tomllib


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", type=Path, required=True)
    parser.add_argument("--resume", type=Path)
    parser.add_argument("--reproduce", action="store_true")
    parser.add_argument("--correction-suffix", action="store_true")
    parser.add_argument("--context-only", action="store_true")
    args = parser.parse_args()
    target = args.target.resolve()
    root = args.resume.resolve() if args.resume else target / "confined-journey" / str(time.time_ns())
    root.mkdir(parents=True, exist_ok=bool(args.resume))
    print(f"confined Serde artifacts: {root}", flush=True)
    environment = os.environ.copy()
    for key in ("RUSTC_BOOTSTRAP", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_TARGET_DIR"):
        environment.pop(key, None)
    environment["RUSTC_WRAPPER"] = "sccache"
    environment["UNDECLARED"] = "native-fixture-value"
    (root / "undeclared.txt").write_text("owned native input\n")
    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
    compiler = sysroot / "bin/rustc.exe"
    environment["PATH"] = str(sysroot / "bin") + os.pathsep + environment["PATH"]
    host = next(line.removeprefix("host: ") for line in subprocess.check_output([str(compiler), "-vV"], text=True).splitlines() if line.startswith("host: "))
    compiler_context = dict(version=subprocess.check_output([str(compiler), "-vV"], text=True), native_target=host, guest_target="wasm32-wasip1", native_cfg=subprocess.check_output([str(compiler), "--print", "cfg", "--target", host], text=True).splitlines(), guest_cfg=subprocess.check_output([str(compiler), "--print", "cfg", "--target", "wasm32-wasip1"], text=True).splitlines(), consumer_cfg=[[],["confined_field"]])
    assert "commit-hash: 4a4ef493e3a1488c6e321570238084b38948f6db" in compiler_context["version"]
    for audience in ("native", "guest"):
        compiler_context[audience+"_pointer_width"] = next(cfg.split("=",1)[1].strip('"') for cfg in compiler_context[audience+"_cfg"] if cfg.startswith("target_pointer_width="))
    linker = sysroot / "lib/rustlib" / host / "bin/rust-lld.exe"
    environment["CARGO_TARGET_" + host.upper().replace("-", "_") + "_LINKER"] = str(linker)
    guest_api = target / "wasm32-wasip1/debug/libproc_macro.rlib"
    assert guest_api.exists()
    home = Path(environment["CARGO_HOME"])

    def run(command, label, env=environment, cwd=root, success=True):
        start = time.monotonic()
        result = subprocess.run([str(c) for c in command], env=env, cwd=cwd, capture_output=True, text=True)
        (root / (label + ".log")).write_text(result.stdout + result.stderr)
        assert (result.returncode == 0) == success, result.stdout + result.stderr
        print(f"{label}: exit={result.returncode}, {time.monotonic()-start:.2f}s", flush=True)
        return result

    guest = root / "guest"
    guest.mkdir(exist_ok=True)
    # The original source package and two entries are explicit recipe inputs.
    archive = next((home / "registry/cache").glob("*/serde_derive-1.0.229.crate"))
    original_digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    assert original_digest == "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            if not member.isfile():
                assert member.isdir()
                continue
            relative = Path(member.name).relative_to("serde_derive-1.0.229")
            assert not relative.is_absolute() and ".." not in relative.parts
            path = guest / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(source.extractfile(member).read())
    original = (guest / "src/lib.rs").read_text()
    effective, count = re.subn(r"#\[proc_macro_derive\((?:Serialize|Deserialize), attributes\(serde\)\)\]\n", "", original)
    assert count == 2
    effective += '''
#[unsafe(no_mangle)]
pub extern "C" fn worth_serialize(input:u32)->u32{derive_serialize(proc_macro::TokenStream::from_host(input)).0}
#[unsafe(no_mangle)]
pub extern "C" fn worth_deserialize(input:u32)->u32{derive_deserialize(proc_macro::TokenStream::from_host(input)).0}
#[unsafe(no_mangle)]
pub extern "C" fn denied_read(input:u32)->u32{let _=std::fs::read("undeclared.txt");input}
#[unsafe(no_mangle)]
pub extern "C" fn denied_environment(input:u32)->u32{let _=std::env::var("UNDECLARED");input}
#[unsafe(no_mangle)]
pub extern "C" fn invalid_handle(_:u32)->u32{let _=proc_macro::TokenStream::from_host(0xffff_ffff).is_empty();0}
#[unsafe(no_mangle)]
pub extern "C" fn work_limit(_:u32)->u32{loop{std::hint::spin_loop();}}
#[unsafe(no_mangle)]
pub extern "C" fn invisible_depth(_:u32)->u32{
let mut stream="x".parse::<proc_macro::TokenStream>().unwrap();
loop{stream=std::iter::once(proc_macro::TokenTree::from(proc_macro::Group::new(proc_macro::Delimiter::None,stream))).collect();}
}
#[unsafe(no_mangle)]
pub extern "C" fn wide_tokens(_:u32)->u32{
let mut stream="x".parse::<proc_macro::TokenStream>().unwrap();
loop{stream.extend(std::iter::once(stream.clone()));}
}
#[unsafe(no_mangle)]
pub extern "C" fn deep_parse(_:u32)->u32{let text="(".repeat(500)+&")".repeat(500);let _=text.parse::<proc_macro::TokenStream>();0}
fn replace_hygiene(stream:proc_macro::TokenStream,caller:&proc_macro::TokenTree)->proc_macro::TokenStream{
    stream.into_iter().map(|mut token|{
        match &mut token{
            proc_macro::TokenTree::Ident(ident) if ident.to_string()=="caller"=>return caller.clone(),
            proc_macro::TokenTree::Ident(ident) if ident.to_string()=="helper"=>ident.set_span(proc_macro::Span::mixed_site()),
            proc_macro::TokenTree::Group(group)=>{let mut replacement=proc_macro::Group::new(group.delimiter(),replace_hygiene(group.stream(),caller));replacement.set_span(group.span());return replacement.into();}
            _=>{}
        }
        token
    }).collect()
}
#[unsafe(no_mangle)]
pub extern "C" fn hygiene(input:u32)->u32{
    let caller=proc_macro::TokenStream::from_host(input).into_iter().next().unwrap();
    replace_hygiene("{let helper=1;caller+helper}".parse().unwrap(),&caller).0
}
#[unsafe(no_mangle)]
pub extern "C" fn fidelity(input:u32)->u32{
let stream=proc_macro::TokenStream::from_host(input);
let mut tokens=stream.clone().into_iter();
let alpha=tokens.next().unwrap();let raw=tokens.next().unwrap();
assert_eq!(alpha.span().source_text().unwrap(),"alpha");
assert_eq!(raw.span().source_text().unwrap(),"r#type");
assert_eq!(alpha.span().join(raw.span()).unwrap().source_text().unwrap(),"alpha r#type");
let proc_macro::TokenTree::Group(group)=tokens.next().unwrap()else{panic!()};
assert_eq!(group.span_open().source_text().unwrap(),"{");
assert_eq!(group.span_close().source_text().unwrap(),"}");
assert!(raw.span().line()>0&&raw.span().column()>0);
let relocated=raw.span().located_at(alpha.span());
assert_eq!(relocated.line(),alpha.span().line());
let _=raw.span().resolved_at(proc_macro::Span::mixed_site());
"pub const FIDELITY:u32=17;".parse::<proc_macro::TokenStream>().unwrap().0
}
'''
    (guest / "src/lib.rs").write_text(effective)
    (guest / "Cargo.toml").write_text('''[package]
name="serde_derive"
version="1.0.229"
edition="2021"
[workspace]
[lib]
name="confined_serde_guest"
crate-type=["cdylib"]
[dependencies]
proc-macro2="=1.0.107"
quote="=1.0.47"
syn={version="=3.0.5",default-features=false,features=["clone-impls","derive","parsing","printing","proc-macro"]}
''')
    run(["cargo", "generate-lockfile", "--offline"], "guest-lock", cwd=guest)
    run(["cargo", "fetch", "--locked"], "guest-fetch", cwd=guest)
    lock = tomllib.loads((guest / "Cargo.lock").read_text())
    locked = {p["name"]: p for p in lock["package"]}
    assert locked["syn"]["version"] == "3.0.5"
    # Independently verify all registry bytes before executing their compiler units.
    verified = []
    for package in lock["package"]:
        if "source" not in package:
            continue
        archive = next((home / "registry/cache").glob(f"*/{package['name']}-{package['version']}.crate"))
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == package["checksum"]
        extracted = next((home / "registry/src").glob(f"*/{package['name']}-{package['version']}"))
        with tarfile.open(archive) as source:
            for member in source.getmembers():
                if member.isfile():
                    relative = Path(member.name).relative_to(extracted.name)
                    expected = source.extractfile(member).read()
                    assert (extracted / relative).read_bytes() == expected
        verified.append([package["name"], package["version"], package["checksum"]])
    guest_environment = environment.copy()
    guest_environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(["--extern", "proc_macro=" + str(guest_api), "-L", "dependency=" + str(guest_api.parent / "deps"), "--remap-path-prefix=" + str(root) + "=/worth/confined-fixture"])
    built = run(["cargo", "build", "--locked", "--offline", "--message-format=json", "-j", "4", "--target", "wasm32-wasip1", "--target-dir", root / "guest-target"], "guest-build", env=guest_environment, cwd=guest)
    messages = [json.loads(line) for line in built.stdout.splitlines() if line.startswith("{")]
    generator_context = [{key:event[key] for key in ("package_id","cfgs","env","linked_libs","linked_paths")} for event in messages if event.get("reason")=="build-script-executed"]
    assert any("proc-macro2" in event["package_id"] and "wrap_proc_macro" in event["cfgs"] for event in generator_context), "locked derive chain did not select live compiler objects"
    wasm = root / "guest-target/wasm32-wasip1/debug/confined_serde_guest.wasm"
    assert wasm.exists()
    if args.reproduce:
        run(["cargo", "build", "--locked", "--offline", "-j", "4", "--target", "wasm32-wasip1", "--target-dir", root / "reproduced-target"], "guest-reproduce", env=guest_environment, cwd=guest)
        reproduced = root / "reproduced-target/wasm32-wasip1/debug/confined_serde_guest.wasm"
        assert wasm.read_bytes() == reproduced.read_bytes(), "same effective inputs did not reproduce identical guest bytes"
    recipe = dict(original_archive=original_digest, original_version="1.0.229", variant="wasip1-live-proc-macro-v1", hash_seed_policy="fixed-16-byte-0x57-v1", effective_source=hashlib.sha256((guest / "src/lib.rs").read_bytes()).hexdigest(), guest_api=hashlib.sha256(guest_api.read_bytes()).hexdigest(), guest=hashlib.sha256(wasm.read_bytes()).hexdigest(), dependencies=verified)
    host_library = target / "debug/libimplementation_edition_macro_host.rlib"
    recipe["host_runtime"] = hashlib.sha256(host_library.read_bytes()).hexdigest()
    recipe["host_source_lock"] = hashlib.sha256((Path(__file__).parent.parent / "Cargo.lock").read_bytes()).hexdigest()
    recipe["compiler_context"] = compiler_context
    recipe["guest_flags"] = guest_environment["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")
    recipe["generator_context"] = generator_context
    recipe["unit_context"] = [{key:event[key] for key in ("package_id","target","features","profile")} for event in messages if event.get("reason")=="compiler-artifact"]
    (root / "effective-variant.json").write_text(json.dumps(recipe, indent=2))
    if args.context_only:
        print("PASS actual guest compiler/build-script/unit context; native proof preserved without rerun", flush=True)
        return
    shim = root / "shim.rs"
    shim.write_text('''extern crate proc_macro;
fn invoke(entry:&str,input:proc_macro::TokenStream)->proc_macro::TokenStream{
    match implementation_edition_macro_host::expand(include_bytes!(r"''' + str(wasm) + '''"),entry,input){
        Ok(output)=>{if entry=="fidelity"{assert!(output.observations().iter().any(|read|matches!(read,implementation_edition_macro_host::SpanRead::SourceText(Some(text)) if text=="alpha r#type")));}output.into_tokens()},Err(error)=>{
            if entry.starts_with("denied_"){assert!(matches!(error,implementation_edition_macro_host::Failure::ForbiddenEffect(_)));}
            if entry=="invalid_handle"{assert!(matches!(error,implementation_edition_macro_host::Failure::Protocol(_)));}
            if entry=="work_limit"{assert!(matches!(error,implementation_edition_macro_host::Failure::WorkLimit));}
            if ["invisible_depth","wide_tokens","deep_parse"].contains(&entry){assert!(matches!(error,implementation_edition_macro_host::Failure::WorkLimit));}
            format!("compile_error!({:?});",error.to_string()).parse().unwrap()
        }
    }
}
#[proc_macro_derive(Serialize,attributes(serde))]
pub fn serialize(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("worth_serialize",input)}
#[proc_macro_derive(Deserialize,attributes(serde))]
pub fn deserialize(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("worth_deserialize",input)}
#[proc_macro]pub fn denied_read(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("denied_read",input)}
#[proc_macro]pub fn denied_environment(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("denied_environment",input)}
#[proc_macro]pub fn invalid_handle(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("invalid_handle",input)}
#[proc_macro]pub fn fidelity(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("fidelity",input)}
#[proc_macro]pub fn hygiene(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("hygiene",input)}
#[proc_macro]pub fn work_limit(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("work_limit",input)}
#[proc_macro]pub fn invisible_depth(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("invisible_depth",input)}
#[proc_macro]pub fn wide_tokens(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("wide_tokens",input)}
#[proc_macro]pub fn deep_parse(input:proc_macro::TokenStream)->proc_macro::TokenStream{invoke("deep_parse",input)}
''')
    run([compiler, "--edition=2024", "--crate-type=proc-macro", "--crate-name=confined_derive", "--extern", "implementation_edition_macro_host=" + str(host_library), "-L", "dependency=" + str(target / "debug/deps"), "-C", "linker=" + str(linker), "--out-dir", root, shim], "trusted-shim")
    runtime = root / "runtime"
    runtime.mkdir(exist_ok=True)
    (runtime / "src").mkdir(exist_ok=True)
    (runtime / "src/lib.rs").write_text("pub fn carrier(){}")
    (runtime / "Cargo.toml").write_text('[package]\nname="derive-runtime"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\nserde="=1.0.229"\nserde_json="=1.0.154"\n')
    run(["cargo", "generate-lockfile", "--offline"], "runtime-lock", cwd=runtime)
    run(["cargo", "build", "--locked", "--offline", "-j", "4", "--target-dir", root / "runtime-target"], "runtime-build", cwd=runtime)
    deps = root / "runtime-target/debug/deps"
    serde = next(deps.glob("libserde-*.rlib"))
    serde_json = next(deps.glob("libserde_json-*.rlib"))
    consumer = root / "consumer.rs"
    consumer.write_text('''use confined_derive::{Serialize,Deserialize};
confined_derive::fidelity!(alpha r#type {9u8});
#[derive(Serialize,Deserialize,Debug,PartialEq)]
enum Mode{Idle,Active{level:u8,label:String}}
#[derive(Serialize,Deserialize,Debug,PartialEq)]
struct Inner{r#type:String,mode:Mode}
#[derive(Serialize,Deserialize,Debug,PartialEq)]
struct Envelope{serial:u32,inner:Inner,values:Vec<i64>,optional:Option<String>,#[cfg(confined_field)]extra:bool,#[cfg(target_pointer_width="64")]native_width:u8}
fn main(){let value=Envelope{serial:91,inner:Inner{r#type:"raw-name".into(),mode:Mode::Active{level:7,label:"nondefault".into()}},values:vec![-3,17],optional:Some("present".into()),#[cfg(confined_field)]extra:true,#[cfg(target_pointer_width="64")]native_width:64};let json=serde_json::to_string(&value).unwrap();assert_eq!(serde_json::from_str::<Envelope>(&json).unwrap(),value);assert_eq!(FIDELITY,17);let helper=17;assert_eq!(confined_derive::hygiene!(helper),18);let r#type=17;assert_eq!(confined_derive::hygiene!(r#type),18);println!("{json}");}
''')
    common = [compiler, "--edition=2024", "--extern", "confined_derive=" + str(root / "confined_derive.dll"), "--extern", "serde=" + str(serde), "--extern", "serde_json=" + str(serde_json), "-L", "dependency=" + str(deps), "-C", "linker=" + str(linker), "--out-dir", root]
    run(common + [consumer], "native-consumer")
    result = run([root / "consumer.exe"], "roundtrip")
    assert '"level":7' in result.stdout and '"type":"raw-name"' in result.stdout
    assert '"native_width":64' in result.stdout and '"extra"' not in result.stdout
    if not args.correction_suffix:
        run(common + ["--cfg", "confined_field", consumer], "native-cfg-consumer")
        result = run([root / "consumer.exe"], "cfg-roundtrip")
        assert '"extra":true' in result.stdout
        relocated = root / "relocated"
        relocated.mkdir(exist_ok=True)
        (relocated / "consumer.rs").write_bytes(consumer.read_bytes())
        run(common + ["--cfg", "confined_field", relocated / "consumer.rs"], "relocated-consumer")
        relocated_result = run([root / "consumer.exe"], "relocated-roundtrip")
        assert relocated_result.stdout == result.stdout
    negatives = [("denied_read", "forbidden ambient guest effect"), ("denied_environment", "forbidden ambient guest effect"), ("invalid_handle", "invalid invocation-scoped macro handle")]
    negatives += [(entry, "macro expansion work limit exceeded") for entry in ("invisible_depth", "wide_tokens", "deep_parse")]
    if not args.correction_suffix:
        negatives.append(("work_limit", "macro expansion work limit exceeded"))
    for entry, expected in negatives:
        consumer.write_text(f"confined_derive::{entry}!();fn main(){{}}")
        result = run(common + [consumer], entry, success=False)
        assert expected in result.stderr, result.stderr
    assert (root / "undeclared.txt").read_text() == "owned native input\n"
    print("PASS locked Serde nondefault nested struct/enum roundtrip, raw names, live join/delimiter/location fidelity, ignored-read/environment and invalid-handle refusals", flush=True)


if __name__ == "__main__":
    main()
