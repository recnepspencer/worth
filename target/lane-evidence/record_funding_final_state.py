"""Capture source identity and exact selected lane counts for final review."""
from pathlib import Path
import hashlib, json, re, subprocess
base = Path("target/lane-evidence")
head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
paths = set(subprocess.check_output(["git", "diff", "--name-only", "--", "crates", "workspaces", "tools/boundary-check/config/certification-construction-callers.txt"], text=True, stderr=subprocess.DEVNULL).splitlines())
paths.update(subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "--", "crates", "workspaces"], text=True).splitlines())
files = {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in sorted(paths) if Path(p).is_file()}
(base / "funding-final-source-state.json").write_text(json.dumps({"review_base_head": head, "files": files}, indent=2) + "\n")
rows = []
for path in sorted(base.glob("funding-final-*.log")):
    text = path.read_text(encoding="utf-8", errors="replace")
    def field(key):
        match = re.search(r"^" + re.escape(key) + r": (.*)$", text, re.M)
        return match.group(1).strip() if match else "RUNNING/UNRECORDED"
    rows.append((path.name, field("HEAD"), field("exit_code"), "/".join(field("tests_" + name) for name in ("passed", "failed", "ignored"))))
text = "# Final funding lane evidence\n\nReview base: `" + head + "`, plus dirty source identified in funding-final-source-state.json. The implementation is frozen while qualification runs. Counts are selected pass/fail/ignored; no ignored test is selected or executed. Full commands, times, exits and dirty status remain in each log.\n\n|Log under target/lane-evidence/|Exit|P/F/I|\n|---|---:|---|\n"
for path, _, exit_code, counts in rows:
    text += "|" + path + "|" + exit_code + "|" + counts + "|\n"
text += "\nGreen reused source-identical evidence: funding-controls-native.log's existing nine Native cases plus shared custody (the Native13 packet itself was10/3/0; do not relabel it all-green). Discarded-history growth, cold Native recapture/index/retry and publication proofs are among those passing names. The moved peer is green in funding-final-native-controls.log; the later byte-only fixture changes do not reach its source. Store and Query formatting logs are reused for unchanged source, not silently rerun.\n"
text += "\nBoundary exception authority: orchestration/m3/prompts/p3_reopen_cold_files.md:31: 'The boundary gate stays red only at your queued BC9002 sites. That is expected until the queue reaches them.' The current user preserves earlier grants and puts that queue after funding and the coordinator-named integration merge. Report the mandatory boundary command's actual exit/findings; do not suppress them or grow the baseline. Any other finding blocks funding.\n"
text += "\nKnown failed attempts: the initial final Native3 packet passed Full/peer but failed the small Head reference because its frame report was not a complete Native preflight footprint. A second frame-peak-plus-retained calibration still omitted transient bills and failed Head alone. Both were court premise errors. The final fixture removes the unrelated editable record from capture and creates/updates it after the controlled read, preserving the original128KiB pressure room,64MiB policy,8/65536 values and every funding/history assertion. The two final byte controls pass2/0/0. No calibration or production padding remains.\n"
(base / "funding-final-lanes.md").write_text(text)
print("Recorded", len(files), "source hashes and", len(rows), "lane headers at", head)
