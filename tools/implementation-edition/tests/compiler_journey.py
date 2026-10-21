"""Real Cargo/compiler variants, grouped in one integration target."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

import compiler_counterexamples
from compiler_cache import isolated_cache, statistics


def run(command, *, expected=0, env=None):
    start = time.monotonic()
    result = subprocess.run(command, capture_output=True, text=True, env=env)
    if result.returncode != expected:
        raise AssertionError(
            f"{command}: expected {expected}, got {result.returncode}\n"
            f"{result.stdout}\n{result.stderr}"
        )
    return result, time.monotonic() - start


def components(target):
    records = {}
    current = json.loads((target / "implementation-edition" / "session.json").read_text())["run"]
    for path in (target / "release" / "deps").glob("*.rlib.implementation.json"):
        stamp = json.loads(path.read_text())
        if stamp["observation"] != current:
            continue
        records[stamp["basis"]["context"]["package"][0]] = stamp["closure"]
    return records


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--observer", type=Path, required=True)
    parser.add_argument("--resume", type=Path, help="reuse completed core input variants; run remaining boundaries only")
    parser.add_argument("--classification-only", action="store_true", help="run only cfg activation and native-source boundaries")
    parser.add_argument("--native-only", action="store_true", help="reuse --resume classification fixture for the native alias boundary only")
    args = parser.parse_args()
    if args.native_only and not args.resume:
        parser.error("--native-only requires an existing --resume fixture")
    fixture = Path(__file__).parent / "fixtures" / "closure"
    observer = args.observer.resolve()
    environment = os.environ.copy()
    environment.pop("CARGO_TARGET_DIR", None)
    environment.pop("CARGO_INCREMENTAL", None)
    environment["RUSTC_WRAPPER"] = shutil.which("sccache")
    temporary = args.resume.resolve() if args.resume else observer.parent.parent / "compiler-journey" / str(time.time_ns())
    temporary.mkdir(parents=True, exist_ok=bool(args.resume))
    print(f"compiler artifacts: {temporary}", flush=True)
    with isolated_cache(temporary, environment) as environment:
        if args.native_only:
            compiler_counterexamples.native_alias(observer, temporary, environment)
        elif args.classification_only:
            compiler_counterexamples.classification(observer, temporary, environment)
        elif args.resume:
            remaining(observer, temporary, environment, temporary / "checkout one", temporary / "target one")
        else:
            journey(observer, temporary, fixture, environment)


def journey(observer, temporary, fixture, environment):
    source = temporary / "checkout one"
    target = temporary / "target one"
    shutil.copytree(fixture, source)
    run(["cargo", "generate-lockfile", "--manifest-path", str(source / "Cargo.toml")], env=environment)

    def build(root=source, output=target, extra=()):
        result, elapsed = run(
            [str(observer), "build", str(root / "Cargo.toml"), str(output),
             "--release", "--workspace", *extra], env=environment
        )
        print(f"observed build: {elapsed:.2f}s", flush=True)
        (temporary / f"build-{time.time_ns()}.log").write_text(result.stdout + result.stderr)
        return result

    build()
    original = components(target)
    assert len(original) == 4, original
    executable = target / "release" / ("edition-neutral.exe" if os.name == "nt" else "edition-neutral")
    run([str(executable)], expected=42)
    cold_stats = statistics(environment)
    assert cold_stats["cache_misses"]["counts"].get("Rust", 0) > 0, cold_stats
    build()
    cached_stats = statistics(environment)
    assert cached_stats["cache_hits"]["counts"].get("Rust", 0) > cold_stats["cache_hits"]["counts"].get("Rust", 0), cached_stats
    (temporary / "cache-miss.json").write_text(json.dumps(cold_stats, indent=2))
    (temporary / "cache-hit.json").write_text(json.dumps(cached_stats, indent=2))
    assert components(target) == original
    run([str(executable)], expected=42)
    print("PASS actual per-invocation sccache miss/hit preserves the observed component", flush=True)
    constant = source / "constant" / "src" / "lib.rs"
    constant.write_text(constant.read_text().replace("= 40", "= 41"))
    build()
    changed = components(target)
    run([str(executable)], expected=43)
    for name in ("edition-constant", "edition-method", "edition-neutral"):
        assert changed[name] != original[name], name
    assert changed["edition-unrelated"] == original["edition-unrelated"]

    seed = source / "neutral" / "seed.txt"
    seed.write_text("2\n")
    build()
    generated = components(target)
    run([str(executable)], expected=44)
    assert generated["edition-neutral"] != changed["edition-neutral"]
    assert generated["edition-method"] == changed["edition-method"]
    assert generated["edition-unrelated"] == changed["edition-unrelated"]
    print("PASS transitive semantic constant and generated input change real output", flush=True)

    constant.write_text(constant.read_text().replace("constant_is_positive", "test_only_renamed"))
    build()
    assert components(target) == generated
    constant.write_bytes(("// ordinary descriptive trivia\n" + constant.read_text()).replace("\n", "\r\n").encode())
    build()
    assert components(target) == generated
    print("PASS production test-only exclusion and ordinary trivia/CRLF stability", flush=True)

    relocated = temporary / "checkout two"
    relocated_target = temporary / "target two"
    shutil.copytree(source, relocated)
    build(relocated, relocated_target)
    assert components(relocated_target) == generated
    print("PASS checkout and target path stability", flush=True)

    remaining(observer, temporary, environment, source, target)


def remaining(observer, temporary, environment, source, target):
    # Cargo may correctly rebuild a plain build when the wrapper changes.
    # Exercise the actual denied boundary: an otherwise valid compiler unit
    # tries to consume a plain extern with no observed construction.
    unobserved = temporary / "unobserved target"
    run(["cargo", "build", "--manifest-path", str(source / "Cargo.toml"),
         "--release", "-p", "edition-constant", "--target-dir", str(unobserved), "-j", "4"], env=environment)
    session_path = target / "implementation-edition" / "session.json"
    session = json.loads(session_path.read_text())
    external = next((unobserved / "release" / "deps").glob("libedition_constant-*.rlib"))
    output = temporary / "unobserved consumer"
    output.mkdir(exist_ok=True)
    context = environment.copy()
    context["WORTH_IMPLEMENTATION_OBSERVER_SESSION"] = str(session_path)
    context["WORTH_IMPLEMENTATION_REOBSERVE"] = session["run"]
    context["CARGO_MANIFEST_DIR"] = str(source / "method")
    context["CARGO_PKG_NAME"] = "edition-method"
    context.pop("OUT_DIR", None)
    result = subprocess.run([str(observer), session["compiler_path"], "--crate-name", "edition_method",
                             "--edition", "2024", str(source / "method" / "src" / "lib.rs"),
                             "--crate-type", "lib", "--emit=dep-info,link", "--out-dir", str(output),
                             "--extern", f"edition_constant={external}"],
                            capture_output=True, text=True, env=context)
    (temporary / "unobserved-extern-refusal.log").write_text(result.stdout + result.stderr)
    assert result.returncode != 0 and "unobserved extern/generator" in result.stderr, result.stderr
    result = subprocess.run([str(observer), session["compiler_path"], "--crate-name", "impostor"],
                            capture_output=True, text=True, env=environment)
    assert result.returncode != 0 and "compiler observation requires its Cargo session" in result.stderr, result.stderr
    print("PASS unobserved extern and missing compiler construction context are refused", flush=True)

    compiler_counterexamples.run(observer, temporary, environment, source)


if __name__ == "__main__":
    main()
