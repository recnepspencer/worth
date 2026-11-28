#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

HOTSPOT_OWNERS = {
    ("commit_delta_matrix", "persisted_single_entity_create"): "durability/authority",
    ("durability_append_matrix", "append_canonical_envelope_fresh_store"): "durability/log/local_store",
    ("durability_append_matrix", "append_canonical_envelope_existing_segment"): "durability/log/local_store",
    ("query_packet_matrix", "connectivity_traversal_cross_partition"): "visibility/materialization/read_records/reader",
    ("query_packet_matrix", "entity_kind_scan_partition_matrix"): "visibility/materialization/read_records/reader",
    ("query_packet_matrix", "explicit_targets_cross_partition"): "visibility/materialization/read_records/reader",
    ("profile_matrix", "certification_core_rich_commit_query_round_trip"): "diagnostics/profile + publication",
    ("profile_matrix", "geometry_kernel_rich_commit_query_round_trip"): "diagnostics/profile + publication",
    ("profile_matrix", "certification_core_zero_diagnostics_commit_query_round_trip"): "diagnostics/profile + publication",
    ("replay_recovery_matrix", "durable_replay_lineage_basis"): "replay/logic/authority",
    ("replay_recovery_matrix", "checkpoint_recover_suffix_replay"): "durability/access + replay/logic/authority",
    ("retention_reclaim_matrix", "snapshot_release_to_reclaimable_entity"): "retention/reclaim surfaces",
    ("workflow_matrix", "trade_correction_analysis_round_trip"): "workflow integration surface",
    ("workflow_matrix", "fintech_intraday_risk_branch_round_trip"): "fintech workflow + observability",
    ("workflow_matrix", "fintech_trade_correction_audit_round_trip"): "fintech workflow + observability",
    ("workflow_matrix", "persisted_recovery_replay_round_trip"): "workflow integration + durability/replay",
    ("workflow_matrix", "retention_release_reclaim_round_trip"): "workflow integration + retention/reclaim",
}


def load_jsonl(path: Path):
    rows = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw:
            continue
        rows.append(json.loads(raw))
    return rows


def row_key(row):
    metric = row.get("metric")
    if metric:
        return (row["suite"], row["case"], metric)
    return (row["suite"], row["case"], None)


def format_delta(current, previous):
    if previous is None:
        return ""
    delta = current - previous
    sign = "+" if delta > 0 else ""
    return f"{sign}{delta:.2f}"


def hotspot_owner(row):
    return HOTSPOT_OWNERS.get((row["suite"], row["case"]), "general runtime surface")


def build_report(summary_rows, compare_rows):
    compare_map = {row_key(row): row for row in compare_rows}

    case_rows = [row for row in summary_rows if "mean_elapsed_micros" in row]
    metric_rows = [row for row in summary_rows if "metric" in row]

    lines = []
    lines.append("# WORTH Relational Perf Summary")
    lines.append("")
    lines.append(f"- Cases: {len(case_rows)}")
    lines.append(f"- Metrics: {len(metric_rows)}")
    lines.append(f"- Compared baseline: {'yes' if compare_rows else 'no'}")
    lines.append("")

    if compare_rows:
        case_regressions = []
        for row in case_rows:
            previous = compare_map.get(row_key(row))
            if not previous:
                continue
            delta_median = row["median_elapsed_micros"] - previous["median_elapsed_micros"]
            if delta_median > 0:
                case_regressions.append((delta_median, row, previous))

        metric_regressions = []
        phase_regressions = []
        packet_regressions = []
        scope_regressions = []
        observability_regressions = []
        profile_regressions = []
        for row in metric_rows:
            previous = compare_map.get(row_key(row))
            if not previous:
                continue
            delta_median = row["median"] - previous["median"]
            metric_name = row["metric"]
            if delta_median > 0:
                metric_regressions.append((delta_median, row, previous))
                if metric_name.endswith("_micros"):
                    phase_regressions.append((delta_median, row, previous))
                if "packet" in metric_name:
                    packet_regressions.append((delta_median, row, previous))
                if "scope" in metric_name:
                    scope_regressions.append((delta_median, row, previous))
                if any(
                    token in metric_name
                    for token in ("diagnostic", "trace", "artifact", "comparison")
                ):
                    observability_regressions.append((delta_median, row, previous))
            if metric_name.startswith("profile_") and row["median"] != previous["median"]:
                profile_regressions.append((abs(delta_median), row, previous))

        lines.append("## Top Regressions")
        lines.append("")
        if case_regressions:
            lines.append("| Suite | Case | Median Delta (us) | Current Median | Baseline Median | Likely Owner |")
            lines.append("| --- | --- | ---: | ---: | ---: | --- |")
            for delta_median, row, previous in sorted(case_regressions, key=lambda item: item[0], reverse=True)[:5]:
                lines.append(
                    "| {suite} | {case} | +{delta} | {current} | {baseline} | {owner} |".format(
                        suite=row["suite"],
                        case=row["case"],
                        delta=delta_median,
                        current=row["median_elapsed_micros"],
                        baseline=previous["median_elapsed_micros"],
                        owner=hotspot_owner(row),
                    )
                )
        else:
            lines.append("No case-level median regressions against the comparison baseline.")

        lines.append("")
        if metric_regressions:
            lines.append("| Suite | Case | Metric | Median Delta | Current Median | Baseline Median | Likely Owner |")
            lines.append("| --- | --- | --- | ---: | ---: | ---: | --- |")
            for delta_median, row, previous in sorted(metric_regressions, key=lambda item: item[0], reverse=True)[:5]:
                lines.append(
                    "| {suite} | {case} | {metric} | +{delta} | {current} | {baseline} | {owner} |".format(
                        suite=row["suite"],
                        case=row["case"],
                        metric=row["metric"],
                        delta=delta_median,
                        current=row["median"],
                        baseline=previous["median"],
                        owner=hotspot_owner(row),
                    )
                )
        else:
            lines.append("No phase-metric median regressions against the comparison baseline.")
        lines.append("")

        lines.append("## Diagnostic Hotspots")
        lines.append("")
        diagnostic_sections = [
            ("Phase regressions", phase_regressions),
            ("Packet inflation", packet_regressions),
            ("Scope inflation", scope_regressions),
            ("Observability inflation", observability_regressions),
            ("Profile drift", profile_regressions),
        ]
        for title, rows in diagnostic_sections:
            lines.append(f"### {title}")
            lines.append("")
            if rows:
                lines.append("| Suite | Case | Metric | Median Delta | Current Median | Baseline Median | Likely Owner |")
                lines.append("| --- | --- | --- | ---: | ---: | ---: | --- |")
                for delta_median, row, previous in sorted(rows, key=lambda item: item[0], reverse=True)[:5]:
                    delta_display = (
                        f"{row['median'] - previous['median']:+d}"
                        if title == "Profile drift"
                        else f"+{delta_median}"
                    )
                    lines.append(
                        "| {suite} | {case} | {metric} | {delta} | {current} | {baseline} | {owner} |".format(
                            suite=row["suite"],
                            case=row["case"],
                            metric=row["metric"],
                            delta=delta_display,
                            current=row["median"],
                            baseline=previous["median"],
                            owner=hotspot_owner(row),
                        )
                    )
            else:
                lines.append(f"No {title.lower()} against the comparison baseline.")
            lines.append("")

        owner_totals = {}
        for delta_median, row, _previous in case_regressions:
            owner = hotspot_owner(row)
            owner_totals[owner] = owner_totals.get(owner, 0) + delta_median
        for delta_median, row, _previous in metric_regressions:
            owner = hotspot_owner(row)
            owner_totals[owner] = owner_totals.get(owner, 0) + delta_median
        lines.append("### Owner Radar")
        lines.append("")
        if owner_totals:
            lines.append("| Likely Owner | Aggregate Median Delta |")
            lines.append("| --- | ---: |")
            for owner, delta in sorted(owner_totals.items(), key=lambda item: item[1], reverse=True)[:5]:
                lines.append(f"| {owner} | +{delta} |")
        else:
            lines.append("No aggregate owner regressions against the comparison baseline.")
        lines.append("")

    lines.append("## Case Summaries")
    lines.append("")
    lines.append("| Suite | Case | Mean (us) | Median (us) | Delta Mean (us) | Delta Median (us) | Samples |")
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for row in sorted(case_rows, key=lambda item: (item["suite"], item["case"])):
        previous = compare_map.get(row_key(row))
        prev_mean = previous.get("mean_elapsed_micros") if previous else None
        prev_median = previous.get("median_elapsed_micros") if previous else None
        lines.append(
            "| {suite} | {case} | {mean:.2f} | {median} | {delta_mean} | {delta_median} | {samples} |".format(
                suite=row["suite"],
                case=row["case"],
                mean=row["mean_elapsed_micros"],
                median=row["median_elapsed_micros"],
                delta_mean=format_delta(row["mean_elapsed_micros"], prev_mean),
                delta_median=format_delta(row["median_elapsed_micros"], prev_median),
                samples=row["samples"],
            )
        )

    if metric_rows:
        lines.append("")
        lines.append("## Metric Summaries")
        lines.append("")
        lines.append("| Suite | Case | Metric | Mean | Median | Delta Mean | Delta Median | Samples |")
        lines.append("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |")
        for row in sorted(metric_rows, key=lambda item: (item["suite"], item["case"], item["metric"])):
            previous = compare_map.get(row_key(row))
            prev_mean = previous.get("mean") if previous else None
            prev_median = previous.get("median") if previous else None
            lines.append(
                "| {suite} | {case} | {metric} | {mean:.2f} | {median} | {delta_mean} | {delta_median} | {samples} |".format(
                    suite=row["suite"],
                    case=row["case"],
                    metric=row["metric"],
                    mean=row["mean"],
                    median=row["median"],
                    delta_mean=format_delta(row["mean"], prev_mean),
                    delta_median=format_delta(row["median"], prev_median),
                    samples=row["samples"],
                )
            )

    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description="Summarize worth-relational perf summary JSONL")
    parser.add_argument("--input", required=True, help="Path to summary JSONL")
    parser.add_argument("--output", required=True, help="Path to markdown report")
    parser.add_argument("--compare", help="Optional prior summary JSONL for delta columns")
    args = parser.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    compare_rows = load_jsonl(Path(args.compare)) if args.compare else []
    summary_rows = load_jsonl(input_path)
    output_path.write_text(build_report(summary_rows, compare_rows), encoding="utf-8")


if __name__ == "__main__":
    main()
