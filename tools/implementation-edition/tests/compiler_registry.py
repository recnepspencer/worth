"""Actual locked serde_core, with isolated extracted copies and retained logs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile
import time
import tomllib


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--observer", type=Path, required=True)
    parser.add_argument("--resume", type=Path, help="reuse this fixture after a concrete owner fix")
    args = parser.parse_args()
    observer = args.observer.resolve()
    root = args.resume.resolve() if args.resume else observer.parent.parent / "serde-journey" / str(time.time_ns())
    root.mkdir(parents=True, exist_ok=bool(args.resume))
    print(f"serde compiler artifacts: {root}", flush=True)
    environment = os.environ.copy()
    environment.pop("CARGO_TARGET_DIR", None)
    environment.pop("CARGO_ENCODED_RUSTFLAGS", None)
    environment.pop("RUSTFLAGS", None)
    home = Path(environment["CARGO_HOME"])
    tool = Path(__file__).parent.parent
    lock = tomllib.loads((tool / "Cargo.lock").read_text())
    selected = next(p for p in lock["package"] if p["name"] == "serde_core")
    assert selected["version"] == "1.0.229"
    vendor = root / "vendor"
    if not args.resume:
        # Cargo metadata resolves these inactive derive dependencies too; they
        # receive no component unless an actual compiler unit participates.
        names = {"serde_core", "serde_derive", "proc-macro2", "quote", "syn", "unicode-ident"}
        for package in lock["package"]:
            if package["name"] not in names:
                continue
            archive = next((home / "registry/cache").glob(f"*/{package['name']}-{package['version']}.crate"))
            assert hashlib.sha256(archive.read_bytes()).hexdigest() == package["checksum"]
            destination = vendor / f"{package['name']}-{package['version']}"
            files = {}
            with tarfile.open(archive) as source:
                for member in source.getmembers():
                    if not member.isfile():
                        assert member.isdir()
                        continue
                    relative = Path(member.name).relative_to(destination.name)
                    assert not relative.is_absolute() and ".." not in relative.parts
                    data = source.extractfile(member).read()
                    path = destination / relative
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(data)
                    files[relative.as_posix()] = hashlib.sha256(data).hexdigest()
            (destination / ".cargo-checksum.json").write_text(json.dumps({"files": files, "package": package["checksum"]}))
        (root / ".cargo").mkdir()
        (root / ".cargo/config.toml").write_text('[source.crates-io]\nreplace-with="fixture"\n[source.fixture]\ndirectory="vendor"\n')
        (root / "src").mkdir()
        (root / "Cargo.toml").write_text('[package]\nname="serde-edition-consumer"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\nserde_core="=1.0.229"\n')
        (root / "src/main.rs").write_text('fn main()->std::process::ExitCode { let value=<u8 as serde_core::Deserialize>::deserialize(serde_core::de::value::U8Deserializer::<serde_core::de::value::Error>::new(17)).unwrap(); std::process::ExitCode::from(value) }\n')
    sysroot = Path(subprocess.run(["rustc", "--print", "sysroot"], capture_output=True, text=True, check=True, env=environment).stdout.strip())
    compiler = sysroot / "bin" / ("rustc.exe" if os.name == "nt" else "rustc")
    version = subprocess.run([str(compiler), "-vV"], capture_output=True, text=True, check=True, env=environment).stdout
    host = next(line.removeprefix("host: ") for line in version.splitlines() if line.startswith("host: "))
    environment["RUSTC"] = str(compiler)
    environment["RUSTC_WRAPPER"] = "sccache"
    if os.name == "nt":
        environment["CARGO_TARGET_" + host.upper().replace("-", "_") + "_LINKER"] = str(sysroot / "lib/rustlib" / host / "bin/rust-lld.exe")
    if not args.resume:
        subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=root, env=environment, check=True, capture_output=True)
    fixture_lock = tomllib.loads((root / "Cargo.lock").read_text())
    assert next(p["checksum"] for p in fixture_lock["package"] if p["name"] == "serde_core") == selected["checksum"]
    target = root / "target"
    start = time.monotonic()
    result = subprocess.run([str(observer), "build", str(root / "Cargo.toml"), str(target), "--offline"], cwd=root, env=environment, capture_output=True, text=True)
    (root / "serde-build.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 0, result.stdout + result.stderr
    executable = target / "debug" / ("serde-edition-consumer.exe" if os.name == "nt" else "serde-edition-consumer")
    assert subprocess.run([str(executable)]).returncode == 17
    record_path = next((target / "debug/build").glob("serde_core-*/out/.implementation-generator.json"))
    record = json.loads(record_path.read_text())
    assert record["queries"][0]["operation"] == "selected-rustc-version"
    assert record["environment"]["CARGO_PKG_VERSION_PATCH"]["value"] == "229"
    assert record["environment"]["TARGET"]["value"] == host
    generated = next(w for w in record["writes"] if Path(w["path"]).name == "private.rs")
    assert "__private229" in Path(generated["path"]).read_text()
    stamp = next((target / "debug/deps").glob("libserde_core-*.rlib.implementation.json"))
    description = json.loads(stamp.read_text())
    assert description["basis"]["context"]["package_content"]["checksum"] == selected["checksum"]
    assert description["basis"]["generator"]["queries"] == record["queries"]
    assert description["basis"]["generator"]["environment"] == record["environment"]
    assert not list((target / "debug/deps").glob("*serde_derive*.implementation.json"))
    print(f"PASS real locked serde_core/build-script/private.rs and consumer output=17 ({time.monotonic()-start:.2f}s)", flush=True)

    session_path = target / "implementation-edition/session.json"
    session = json.loads(session_path.read_text())
    query = environment.copy()
    query.update({"WORTH_IMPLEMENTATION_OBSERVER_SESSION":str(session_path), "WORTH_IMPLEMENTATION_REOBSERVE":session["run"], "CARGO_MANIFEST_DIR":record["root"], "OUT_DIR":record["out"], "CARGO_PKG_VERSION_PATCH":"229", "TARGET":host, "RUSTC":str(observer)})
    dispatcher = Path(record["generator"]["artifact"]).parent / ("build-script-build.exe" if os.name == "nt" else "build-script-build")
    result = subprocess.run([str(dispatcher)], cwd=record["root"], env=query, capture_output=True, text=True)
    (root / "query-refusal.log").write_text(result.stdout + result.stderr)
    assert result.returncode != 0 and "generator compiler query differs from selected compiler" in result.stderr, result.stderr
    print("PASS actual generator execution rejects a mismatched compiler query", flush=True)

    extracted = vendor / "serde_core-1.0.229/src/lib.rs"
    original = extracted.read_bytes()
    extracted.write_bytes(original + b"\n// isolated tamper\n")
    checksum_path = extracted.parent.parent / ".cargo-checksum.json"
    checksum = json.loads(checksum_path.read_text())
    checksum["files"]["src/lib.rs"] = hashlib.sha256(extracted.read_bytes()).hexdigest()
    checksum_path.write_text(json.dumps(checksum))
    try:
        result = subprocess.run([str(observer), "build", str(root / "Cargo.toml"), str(target), "--offline"], cwd=root, env=environment, capture_output=True, text=True)
        (root / "checksum-refusal.log").write_text(result.stdout + result.stderr)
        assert result.returncode != 0 and "extracted external content differs from locked archive/tree" in result.stderr, result.stderr
    finally:
        extracted.write_bytes(original)
        checksum["files"]["src/lib.rs"] = hashlib.sha256(original).hexdigest()
        checksum_path.write_text(json.dumps(checksum))
    print("PASS isolated registry tampering is refused despite matching extracted checksum claims", flush=True)


if __name__ == "__main__":
    main()
