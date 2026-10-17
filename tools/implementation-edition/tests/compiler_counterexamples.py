"""Narrow real-compiler counterparts for argument, span and generator defects."""
import json
import os
from pathlib import Path
import subprocess
import time


def run(observer, directory, environment, source):
    root = directory / "counterexamples"
    (root / "src").mkdir(parents=True, exist_ok=True)
    (root / "src" / "main.rs").write_text("fn main() {}\n")
    (root / "Cargo.toml").write_text(
        '[package]\nname="edition-counterexample"\nversion="0.1.0"\nedition="2024"\n[workspace]\n'
    )
    subprocess.run(["cargo", "generate-lockfile", "--manifest-path", str(root / "Cargo.toml")],
                   check=True, capture_output=True, env=environment)
    target = directory / "counterexample target"
    executable = target / "release" / ("edition-counterexample.exe" if os.name == "nt" else "edition-counterexample")
    main = root / "src" / "main.rs"

    def build(env=environment, refusal=None):
        start = time.monotonic()
        result = subprocess.run([str(observer), "build", str(root / "Cargo.toml"), str(target), "--release"],
                                capture_output=True, text=True, env=env)
        (directory / f"counterexample-{time.time_ns()}.log").write_text(result.stdout + result.stderr)
        if refusal:
            assert result.returncode != 0 and refusal in result.stderr, result.stderr
            return None
        assert result.returncode == 0, result.stderr
        print(f"counterexample compiler build: {time.monotonic() - start:.2f}s", flush=True)
        stamp = json.loads(Path(str(executable) + ".implementation.json").read_text())
        return stamp["closure"]

    # Both selectors and both cfg attributes are valid Rust string literals.
    main.write_text(
        r'#[cfg(select="a\\b")] const VALUE:u8=17;' + '\n' +
        r'#[cfg(select="a//b")] const VALUE:u8=23;' + '\n' +
        'fn main()->std::process::ExitCode { std::process::ExitCode::from(VALUE) }\n'
    )
    backslash = environment.copy()
    backslash["CARGO_ENCODED_RUSTFLAGS"] = "--cfg\x1f" + r'select="a\\b"'
    slash = environment.copy()
    slash["CARGO_ENCODED_RUSTFLAGS"] = "--cfg\x1f" + r'select="a//b"'
    first = build(backslash)
    assert subprocess.run([str(executable)]).returncode == 17
    second = build(slash)
    assert subprocess.run([str(executable)]).returncode == 23
    assert first != second
    print("PASS valid escaped-backslash/slash cfg selectors differ in output and edition", flush=True)

    main.write_text('#[cfg(test)] mod tests {}\nconst VALUE:u8=line!() as u8;\n'
                    'fn main()->std::process::ExitCode { std::process::ExitCode::from(VALUE) }\n')
    first = build()
    assert subprocess.run([str(executable)]).returncode == 2
    main.write_text(main.read_text().replace("mod tests {}", "mod tests {\n}"))
    second = build()
    assert subprocess.run([str(executable)]).returncode == 3
    assert first != second
    main.write_text(main.read_text().replace("mod tests", "mod renamed_tests"))
    assert build() == second
    assert subprocess.run([str(executable)]).returncode == 3

    prefix = '#[cfg(test)] mod tests {} const VALUE:u8='
    main.write_text(prefix + 'column!() as u8;\n'
                    'fn main()->std::process::ExitCode { std::process::ExitCode::from(VALUE) }\n')
    first = build()
    assert subprocess.run([str(executable)]).returncode == len(prefix) + 1
    main.write_text(main.read_text().replace("mod tests", "mod longer_tests"))
    second = build()
    assert subprocess.run([str(executable)]).returncode == len(prefix.replace("tests", "longer_tests")) + 1
    assert first != second
    print("PASS compiled line/column effects survive excluded tests; position-neutral edit stays stable", flush=True)

    native = environment.copy()
    native["CARGO_ENCODED_RUSTFLAGS"] = "-Clink-arg=unqualified.lib"
    build(native, refusal="native/input-bearing codegen option requires input qualification: link-arg")
    print("PASS unqualified native link input is refused at its own boundary", flush=True)

    # The static read/declaration sets match, but this declaration never emits.
    script = source / "neutral" / "build.rs"
    original = script.read_text()
    script.write_text(original.replace('println!("cargo::rerun-if-changed=seed.txt");',
                                       'if false { println!("cargo::rerun-if-changed=seed.txt"); }'))
    result = subprocess.run([str(observer), "build", str(source / "Cargo.toml"),
                             str(directory / "generator negative target"), "--release", "-p", "edition-neutral"],
                            capture_output=True, text=True, env=environment)
    (directory / "generator-coverage-refusal.log").write_text(result.stdout + result.stderr)
    script.write_text(original)
    assert result.returncode != 0 and "generator emitted directives omit validated file read" in result.stderr, result.stderr
    print("PASS emitted generator coverage must contain every validated read", flush=True)


def classification(observer, directory, environment):
    root = directory / "classification"
    (root / "src").mkdir(parents=True)
    (root / "Cargo.toml").write_text(
        '[package]\nname="edition-classification"\nversion="0.1.0"\nedition="2024"\n[workspace]\n'
    )
    main = root / "src" / "main.rs"
    main.write_text(
        '#[cfg(test)] const VALUE:u8=17;\n'
        '#[cfg(not(test))] const VALUE:u8=5;\n'
        'extern "C" fn rust_defined()->u8 { VALUE }\n'
        'fn main()->std::process::ExitCode { std::process::ExitCode::from(rust_defined()) }\n'
    )
    subprocess.run(["cargo", "generate-lockfile", "--manifest-path", str(root / "Cargo.toml")],
                   capture_output=True, check=True, env=environment)
    target = directory / "classification target"
    name = "edition-classification.exe" if os.name == "nt" else "edition-classification"
    executable = target / "release" / name

    def build(env, expected):
        start = time.monotonic()
        result = subprocess.run([str(observer), "build", str(root / "Cargo.toml"), str(target), "--release"],
                                capture_output=True, text=True, env=env)
        (directory / f"classification-{time.time_ns()}.log").write_text(result.stdout + result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr
        assert subprocess.run([str(executable)]).returncode == expected
        print(f"classification compiler build: {time.monotonic() - start:.2f}s output={expected}", flush=True)
        stamp = json.loads(Path(str(executable) + ".implementation.json").read_text())
        return stamp["closure"], stamp["context_digest"]

    ordinary = environment.copy()
    ordinary.pop("CARGO_ENCODED_RUSTFLAGS", None)
    ordinary.pop("RUSTFLAGS", None)
    inactive = build(ordinary, 5)
    enabled = ordinary.copy()
    enabled["CARGO_ENCODED_RUSTFLAGS"] = "--cfg\x1ftest"
    first = build(enabled, 17)
    main.write_text(main.read_text().replace("VALUE:u8=17", "VALUE:u8=23"))
    enabled["CARGO_ENCODED_RUSTFLAGS"] = "--cfg=test"
    second = build(enabled, 23)
    assert first[0] != second[0]
    assert first[1] == second[1], "cfg spelling must not confound the semantic constant edit"
    assert build(ordinary, 5) == inactive
    print("PASS --cfg test retains compiled constants; inactive tests stay stable; Rust-defined extern ABI works", flush=True)

    native_alias(observer, directory, ordinary)


def native_alias(observer, directory, environment):
    root = directory / "classification"
    main = root / "src" / "main.rs"
    target = directory / "classification target"
    name = "edition-classification.exe" if os.name == "nt" else "edition-classification"
    ordinary = environment.copy()
    ordinary.pop("CARGO_ENCODED_RUSTFLAGS", None)
    ordinary.pop("RUSTFLAGS", None)

    main.write_text(
        'use core::arch::r#asm as emit;\n'
        '#[unsafe(no_mangle)] pub unsafe extern "C" fn embedded() {\n'
        '    unsafe { emit!(".incbin \\\"payload.bin\\\"", options(noreturn)); }\n'
        '}\n'
        'fn main()->std::process::ExitCode { std::process::ExitCode::from(5) }\n'
    )
    (root / "payload.bin").write_bytes(bytes([0x90, 0xC3]))
    plain_target = directory / "plain native target"
    plain = subprocess.run(["cargo", "build", "--manifest-path", str(root / "Cargo.toml"), "--release",
                            "--target-dir", str(plain_target), "-j", "4"],
                           capture_output=True, text=True, env=ordinary)
    (directory / "native-alias-plain.log").write_text(plain.stdout + plain.stderr)
    assert plain.returncode == 0, plain.stdout + plain.stderr
    assert subprocess.run([str(plain_target / "release" / name)]).returncode == 5
    result = subprocess.run([str(observer), "build", str(root / "Cargo.toml"), str(target), "--release"],
                            capture_output=True, text=True, env=ordinary)
    (directory / "native-alias-refusal.log").write_text(result.stdout + result.stderr)
    assert result.returncode != 0 and "native source inputs require input qualification" in result.stderr, result.stderr
    print("PASS raw aliased asm/incbin compiles ordinarily and is refused by compiler observation", flush=True)
