"""Real Rust escape control, pre-execution refusal, and supported typed seed."""
import argparse
import os
from pathlib import Path
import subprocess
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--observer", type=Path, required=True)
    args = parser.parse_args()
    observer = args.observer.resolve()
    root = observer.parent.parent / "effect-correction" / str(time.time_ns())
    root.mkdir(parents=True)
    print(f"effect correction artifacts: {root}", flush=True)
    environment = os.environ.copy()
    for key in ("CARGO_TARGET_DIR", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        environment.pop(key, None)
    sysroot = Path(subprocess.run(["rustc", "--print", "sysroot"], env=environment, capture_output=True, text=True, check=True).stdout.strip())
    compiler = sysroot / "bin" / ("rustc.exe" if os.name == "nt" else "rustc")
    version = subprocess.run([str(compiler), "-vV"], env=environment, capture_output=True, text=True, check=True).stdout
    host = next(line.removeprefix("host: ") for line in version.splitlines() if line.startswith("host: "))
    environment["RUSTC"] = str(compiler)
    environment["RUSTC_WRAPPER"] = "sccache"
    if os.name == "nt":
        environment["CARGO_TARGET_" + host.upper().replace("-", "_") + "_LINKER"] = str(sysroot / "lib/rustlib" / host / "bin/rust-lld.exe")
    fixture = Path(__file__).parent / "fixtures"
    (root / "Cargo.toml").write_text('[package]\nname="effect-correction"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
    (root / "src").mkdir()
    (root / "src/main.rs").write_text("fn main(){}\n")
    (root / "build.rs").write_bytes((fixture / "generator_escape.rs").read_bytes())
    subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=root, env=environment, check=True, capture_output=True)

    def run(arguments, log):
        start = time.monotonic()
        result = subprocess.run(arguments, cwd=root, env=environment, capture_output=True, text=True)
        (root / log).write_text(result.stdout + result.stderr)
        return result, time.monotonic() - start

    plain = root / "plain-target"
    result, elapsed = run(["cargo", "build", "--locked", "--offline", "-j", "4", "--target-dir", str(plain)], "plain-control.log")
    assert result.returncode == 0, result.stderr
    proofs = list((plain / "debug/build").glob("effect-correction-*/proof"))
    inside = list((plain / "debug/build").glob("effect-correction-*/out/proof"))
    assert len(proofs) == len(inside) == 1
    assert proofs[0].read_bytes() == inside[0].read_bytes() == b"x"
    print(f"PASS plain Cargo executes valid u8-overflow branch and both writes ({elapsed:.2f}s)", flush=True)

    target = root / "observed-target"
    command = [str(observer), "build", str(root / "Cargo.toml"), str(target), "--offline"]
    result, elapsed = run(command, "pre-execution-refusal.log")
    assert result.returncode != 0 and "binary operator or mutation" in result.stderr, result.stderr
    assert not list(target.rglob("proof")), "refused generator executed a write"
    assert not list(target.rglob("*.generator.exe")) and not list(target.rglob("*.generator"))
    print(f"PASS observer refuses escape before native generator compilation/execution; no proof file ({elapsed:.2f}s)", flush=True)

    seed = (fixture / "closure/neutral/build.rs").read_text()
    # Keep the original inferred-u8 seed and distinguish explicit-u8 overflow
    # from the former u32 interpretation in the actual compiled positive.
    seed = seed.replace("fn main() {", 'fn main() { if "256".parse::<u8>().ok() != None { return; }', 1)
    (root / "build.rs").write_text(seed)
    (root / "seed.txt").write_text("17\n")
    (root / "src/main.rs").write_text('include!(concat!(env!("OUT_DIR"),"/generated.rs")); fn main()->std::process::ExitCode {std::process::ExitCode::from(GENERATED)}\n')
    result, elapsed = run(command, "supported-seed.log")
    assert result.returncode == 0, result.stdout + result.stderr
    executable = target / "debug" / ("effect-correction.exe" if os.name == "nt" else "effect-correction")
    assert subprocess.run([str(executable)]).returncode == 17
    print(f"PASS actual inferred-u8 seed plus explicit-u8 overflow produces consumer value 17 ({elapsed:.2f}s)", flush=True)


if __name__ == "__main__":
    main()
