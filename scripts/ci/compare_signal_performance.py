#!/usr/bin/env python3
"""Capture and adjudicate complete matched Signal performance families.

Capture order is exactly A1,B1,B2,A2. Capture and two-file compare are never
acceptance; only a complete matched adjudication may emit performance_pass.
Matched exit: 0 pass, 1 relative regression, 2 invalid, 3 benchmark failure,
4 absolute violation, 5 inconclusive noise. Single compare retains exits 0/1/2/4.
Peak is measured-group requested-object high-water, not RSS or ordinary timing.
"""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

from signal_performance_comparison import compare, compare_matched
from signal_performance_protocol import (
    ACCESS_COUNTERS,
    CAPTURE_ORDER,
    CAPTURE_VERSION,
    CASE_RESOLUTION,
    CASES,
    CONTRACTS,
    FAMILY,
    PROBES,
    SCOPED_ALLOCATION_METRICS,
    TESTS,
    WORKLOAD_WARMUPS,
    case_protocol,
    configuration,
    decode,
    environment,
    expected_budgets,
    matched_protocol,
    new_output,
    require,
    validate_cases,
    validate_capture,
)


def require_listing(text):
    listed = [line[:-6] for line in text.splitlines() if line.endswith(": test")]
    require(listed == TESTS, "ignored test listing differs from reviewed whole-family roster")


def require_completion(text):
    summaries = re.findall(
        r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text
    )
    require(summaries == [("ok", str(len(TESTS)), "0", "0")],
            "benchmark crashed, skipped cases or has no complete success summary")


def capture(args):
    root = args.root.resolve()
    roots = [root, Path(__file__).resolve().parents[2]]
    output = new_output(args.output, roots)
    require(args.target_dir.is_absolute()
            and not any(args.target_dir.resolve().is_relative_to(source) for source in roots),
            "target-dir must be outside source")
    context = environment(root)
    paths = {
        probe: new_output(Path(str(output) + f".{probe}.jsonl"), roots)
        for probe in PROBES
    }
    log_path = new_output(Path(str(output) + ".log"), roots)
    matched_capture = {
        "set_id": args.matched_set,
        "slot": args.slot,
        "sequence_index": CAPTURE_ORDER.index(args.slot),
        "started_unix_nanos": time.time_ns(),
    }
    data = dict(
        version=CAPTURE_VERSION,
        status="benchmark_failed",
        measurement_protocol=matched_protocol(),
        matched_capture=matched_capture,
        configuration=configuration(),
        environment=context,
        test_roster=TESTS,
        cases={},
        commands={},
    )
    with output.open("x", encoding="utf-8") as destination, \
            log_path.open("x", encoding="utf-8") as log:
        try:
            for probe in PROBES:
                _capture_probe(args, root, paths[probe], log_path, log, data, probe)
            data["status"] = "captured"
            validate_capture(data, args.slot)
        except (ValueError, OSError, KeyError, TypeError, IndexError, AttributeError) as error:
            data["status"] = "benchmark_failed"
            data["error"] = str(error)
        json.dump(data, destination, indent=2, allow_nan=False)
    print(f"{data['status']}: {output} (capture alone is not a performance pass)")
    return 0 if data["status"] == "captured" else 3


def _capture_probe(args, root, probe_path, log_path, log, data, probe):
    features = "profile-extended" + (",test-peak-allocation" if probe == "peak" else "")
    command = [
        "cargo", "test", "--locked", "--release", "--manifest-path", str(root / "Cargo.toml"),
        "--target-dir", str(args.target_dir), "-p", "worth-signal", "--lib",
        "--no-default-features", "--features", features, FAMILY, "--",
    ]
    listing = subprocess.run(
        command + ["--ignored", "--list", "--format=terse"], cwd=root,
        text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    )
    log.write(listing.stdout)
    require(listing.returncode == 0, f"{probe} build/list failed ({listing.returncode})")
    require_listing(listing.stdout)
    run_command = command + [
        "--ignored", "--test-threads=1", "--nocapture", "--color=never"
    ]
    print(f"[{probe}] {subprocess.list2cmdline(run_command)}", flush=True)
    log.write(json.dumps(run_command) + "\n")
    log.flush()
    probe_environment = dict(os.environ, WORTH_SIGNAL_PERF_OUTPUT=str(probe_path))
    result = subprocess.run(
        run_command, cwd=root, env=probe_environment, stdout=log, stderr=subprocess.STDOUT
    )
    log.flush()
    data["commands"][probe] = dict(argv=run_command, returncode=result.returncode)
    require(result.returncode == 0,
            f"{probe} benchmark failed ({result.returncode}); see {log_path}")
    text = log_path.read_text(encoding="utf-8")
    require_completion(text[text.rfind(json.dumps(run_command)):])
    records = [decode(line) for line in probe_path.read_text().splitlines()]
    data["cases"][probe] = validate_cases(records, probe)


def _write_comparison(output_path, report):
    with output_path.open("x", encoding="utf-8") as destination:
        json.dump(report, destination, indent=2, allow_nan=False)


def _single_comparison(args):
    require(args.baseline.resolve() != args.candidate.resolve(),
            "baseline and candidate must be independent files")
    report, code = compare(
        decode(args.baseline.read_text()), decode(args.candidate.read_text())
    )
    _write_comparison(args.output, report)
    print(f"{report['status']}: relative={report['relative_verdict']}; "
          f"absolute={report['absolute_verdict']}; {args.output}")
    return code


def _matched_comparison(args):
    inputs = [args.a1, args.b1, args.b2, args.a2]
    require(len({path.resolve() for path in inputs}) == len(inputs),
            "all four matched slots require independent files")
    captures = [decode(path.read_text()) for path in inputs]
    report, code = compare_matched(*captures)
    _write_comparison(args.output, report)
    print(f"{report['status']}: relative={report['relative_verdict']}; "
          f"absolute={report['absolute_verdict']}; {args.output}")
    return code


def _parser():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    commands = parser.add_subparsers(dest="action", required=True)
    record = commands.add_parser("capture")
    record.add_argument("--root", type=Path, required=True)
    record.add_argument("--target-dir", type=Path, required=True)
    record.add_argument("--output", type=Path, required=True)
    record.add_argument("--matched-set", required=True)
    record.add_argument("--slot", choices=CAPTURE_ORDER, required=True)
    comparison = commands.add_parser("compare")
    comparison.add_argument("--baseline", type=Path, required=True)
    comparison.add_argument("--candidate", type=Path, required=True)
    comparison.add_argument("--output", type=Path, required=True)
    matched = commands.add_parser("matched")
    for slot in ("a1", "b1", "b2", "a2"):
        matched.add_argument(f"--{slot}", type=Path, required=True)
    matched.add_argument("--output", type=Path, required=True)
    return parser


def main():
    args = _parser().parse_args()
    try:
        if args.action == "capture":
            return capture(args)
        args.output = new_output(args.output, [Path(__file__).resolve().parents[2]])
        if args.action == "compare":
            return _single_comparison(args)
        return _matched_comparison(args)
    except (ValueError, OSError, KeyError, TypeError, IndexError, AttributeError,
            subprocess.SubprocessError) as error:
        print(f"invalid measurement input/posture: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
