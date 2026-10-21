"""Stage checksum-verified pinned compiler components in owned target storage.

Never installs into rustup or changes shared toolchain/cache configuration.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import urllib.request

PIN = "1.94.0-x86_64-pc-windows-msvc"
DATE = "2026-03-05"
ARCHIVES = {
    "rustc-dev": "356d7347b613f37db2f979e27b6f3d37246aa26c9e2a300e6ff5f06cc314e53e",
    "rustc": "e0f59503778b563d3ee6d4edecd52b44056779ffe456181bad1fb4dc069802e6",
    "rust-std": "f8e2b43e52957e8224d97de68d6ec5da01613eb98bce4199de80071678b5fda0",
}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--storage", type=Path, required=True)
    parser.add_argument("--driver-dir", type=Path, required=True)
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[3]
    storage = args.storage.resolve()
    driver = args.driver_dir.resolve()
    assert storage.is_relative_to(repository / "target")
    assert driver.is_relative_to(repository / "target")
    storage.mkdir(parents=True, exist_ok=True)
    driver.mkdir(parents=True, exist_ok=True)
    for component, expected in ARCHIVES.items():
        filename = f"{component}-{PIN}.tar.xz"
        archive = storage / filename
        url = f"https://static.rust-lang.org/dist/{DATE}/{filename}"
        if not archive.exists():
            urllib.request.urlretrieve(url, archive)
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == expected, filename
        destination = storage / f"{component}-{PIN}"
        verification_root = destination
        if component == "rustc-dev":
            # Compiler artifacts participate in the driver build. Rust source
            # bundled for debugging is not an input to this prebuilt compiler.
            verification_root /= "rustc-dev/lib/rustlib/x86_64-pc-windows-msvc/lib"
        member_prefix = verification_root.relative_to(storage).as_posix() + "/"
        if not destination.exists():
            with tarfile.open(archive) as source:
                # Component archives are trusted by checksum, but extraction
                # still refuses escaping or linked paths explicitly.
                for member in source.getmembers():
                    path = storage / member.name
                    assert path.resolve().is_relative_to(storage)
                    assert member.isdir() or member.isfile()
                source.extractall(storage, members=[m for m in source.getmembers()
                    if m.name.startswith(member_prefix)], filter="data")
        # Cached development metadata is a compiler input too. A valid archive
        # alone cannot bless modified extracted libraries used to link a driver.
        expected_files = set()
        with tarfile.open(archive, mode="r|xz") as source:
            for member in source:
                if not member.isfile() or not member.name.startswith(member_prefix):
                    continue
                path = storage / member.name
                assert path.resolve().is_relative_to(verification_root)
                assert not path.is_symlink() and path.is_file(), member.name
                assert path.read_bytes() == source.extractfile(member).read(), member.name
                expected_files.add(path.relative_to(verification_root))
        actual_files = set()
        for path in verification_root.rglob("*"):
            assert not path.is_symlink(), str(path)
            if path.is_file():
                actual_files.add(path.relative_to(verification_root))
            else:
                assert path.is_dir(), str(path)
        assert actual_files == expected_files, f"unexpected extracted {component} files"
        print(f"verified {component}: {expected}", flush=True)
    binary = storage / f"rustc-{PIN}/rustc/bin"
    evidence = {"pin": PIN, "date": DATE, "archives": ARCHIVES, "runtime": {}}
    for name in ("rustc_driver-eab2645d391441c2.dll", "std-2137bdd3874dafb5.dll"):
        source = binary / name
        # Read expected bytes from the verified archive, rather than trusting
        # a previously extracted directory or adjacent user-supplied binary.
        with tarfile.open(storage / f"rustc-{PIN}.tar.xz") as archive:
            expected_bytes = archive.extractfile(f"rustc-{PIN}/rustc/bin/{name}").read()
        assert source.read_bytes() == expected_bytes
        shutil.copyfile(source, driver / name)
        evidence["runtime"][name] = hashlib.sha256(expected_bytes).hexdigest()
    (driver / "compiler-provenance.json").write_text(json.dumps(evidence, indent=2))
    print(json.dumps(evidence["runtime"], indent=2), flush=True)
    sysroot = driver / "pinned-sysroot"
    records = []
    # sccache discovers and hashes the compiler's DLLs in sysroot/bin.
    (sysroot / "bin").mkdir(parents=True, exist_ok=True)
    for name, digest in evidence["runtime"].items():
        shutil.copyfile(driver / name, sysroot / "bin" / name)
        records.append(("bin/" + name, digest))
    prefix = f"rust-std-{PIN}/rust-std-x86_64-pc-windows-msvc/"
    with tarfile.open(storage / f"rust-std-{PIN}.tar.xz") as archive:
        for member in archive.getmembers():
            if not member.isfile() or not member.name.startswith(prefix + "lib/"):
                continue
            relative = Path(member.name.removeprefix(prefix))
            content = archive.extractfile(member).read()
            path = sysroot / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            records.append((relative.as_posix(), hashlib.sha256(content).hexdigest()))
    basis = "".join(name + "\0" + digest + "\n" for name, digest in sorted(records))
    print("pinned sysroot tree: " + hashlib.sha256(basis.encode()).hexdigest(), flush=True)


if __name__ == "__main__":
    main()
