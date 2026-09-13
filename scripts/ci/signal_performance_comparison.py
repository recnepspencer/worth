"""Single-pair and counterbalanced matched Signal performance adjudication."""

import math
from fractions import Fraction

from signal_performance_protocol import (
    CAPTURE_ORDER,
    PEAK,
    PROBES,
    case_config,
    require,
    summarize,
    validate_capture,
)


def _metric_applies_to_probe(probe, metric):
    return (probe == "peak") == (metric == PEAK)


def _finite_ratio(observed, baseline):
    if baseline == 0:
        return Fraction(1, 1) if observed == 0 else None
    return Fraction(observed, baseline)


def _absolute_violations(captures):
    violations = []
    for slot, capture in captures.items():
        for case in capture["cases"]["ordinary"]:
            values = summarize(case)
            key = "|".join(case["contract"][name] for name in ("suite", "profile", "executor"))
            for counter, maximum in case["contract"]["access_counter_maxima"]:
                observed = values[f"metrics.access_counters.{counter}"]["max"]
                if observed > maximum:
                    violations.append(dict(slot=slot, probe="ordinary", case=key,
                                           counter=counter, observed=observed, maximum=maximum))
    return violations


def compare(baseline, candidate):
    """Compare two directions diagnostically; A/A is required for acceptance."""
    validate_capture(baseline)
    validate_capture(candidate)
    require(baseline["environment"] == candidate["environment"],
            "environment mismatch; cannot compare")
    report = {"acceptance_posture": "diagnostic_only_requires_full_matched_aa",
              "performance_acceptance": False, "relative_regressions": [],
              "absolute_contract_violations": [], "distributions": {}}
    for probe in PROBES:
        for before, after in zip(baseline["cases"][probe], candidate["cases"][probe]):
            require(case_config(before) == case_config(after),
                    "sampling/contract/budget mismatch")
            old, new = summarize(before), summarize(after)
            require(old.keys() == new.keys(), "baseline/candidate metric roster mismatch")
            key = "|".join(after["contract"][name] for name in ("suite", "profile", "executor"))
            report["distributions"][f"{probe}|{key}"] = {"baseline": old, "candidate": new}
            for metric, budgets in after["relative_budgets"].items():
                if not _metric_applies_to_probe(probe, metric):
                    continue
                for stat, budget in budgets.items():
                    expected, observed = old[metric][stat], new[metric][stat]
                    allowed = math.ceil(expected * budget)
                    if observed > allowed:
                        report["relative_regressions"].append(
                            dict(case=key, probe=probe, metric=metric, statistic=stat,
                                 baseline=expected, candidate=observed, allowed=allowed,
                                 budget=budget)
                        )
    report["absolute_contract_violations"] = _absolute_violations(
        {"baseline": baseline, "candidate": candidate}
    )
    report["relative_verdict"] = (
        "regression" if report["relative_regressions"] else "within budgets"
    )
    report["absolute_verdict"] = (
        "violation" if report["absolute_contract_violations"] else "within contracts"
    )
    code = (1 if report["relative_regressions"] else
            4 if report["absolute_contract_violations"] else 0)
    report["status"] = "diagnostic_within_budgets" if code == 0 else "rejected"
    return report, code


def compare_matched(a1, b1, b2, a2):
    captures = {"A1": a1, "B1": b1, "B2": b2, "A2": a2}
    for slot in CAPTURE_ORDER:
        validate_capture(captures[slot], slot)
    set_ids = {capture["matched_capture"]["set_id"] for capture in captures.values()}
    require(len(set_ids) == 1, "matched captures belong to different sets")
    starts = [captures[slot]["matched_capture"]["started_unix_nanos"] for slot in CAPTURE_ORDER]
    require(starts == sorted(starts) and len(set(starts)) == len(starts),
            "capture start times do not follow the frozen A1,B1,B2,A2 order")
    environments = [captures[slot]["environment"] for slot in CAPTURE_ORDER]
    require(all(environment == environments[0] for environment in environments[1:]),
            "environment mismatch; cannot compare matched captures")

    report = {
        "measurement_protocol": a1["measurement_protocol"],
        "matched_set": next(iter(set_ids)),
        "capture_slots": list(CAPTURE_ORDER),
        "relative_regressions": [],
        "inconclusive_noise": [],
        "resolved_relative_claims": [],
        "absolute_contract_violations": _absolute_violations(captures),
        "distributions": {},
    }
    for probe in PROBES:
        case_count = len(a1["cases"][probe])
        for case_index in range(case_count):
            cases = {slot: captures[slot]["cases"][probe][case_index] for slot in CAPTURE_ORDER}
            reference_config = case_config(cases["A1"])
            require(all(case_config(case) == reference_config for case in cases.values()),
                    "sampling/contract/budget mismatch")
            summaries = {slot: summarize(case) for slot, case in cases.items()}
            reference_metrics = summaries["A1"].keys()
            require(all(summary.keys() == reference_metrics for summary in summaries.values()),
                    "matched metric roster mismatch")
            contract = cases["A1"]["contract"]
            key = "|".join(contract[name] for name in ("suite", "profile", "executor"))
            report["distributions"][f"{probe}|{key}"] = summaries
            for metric, budgets in cases["A1"]["relative_budgets"].items():
                if not _metric_applies_to_probe(probe, metric):
                    continue
                for statistic, budget in budgets.items():
                    _adjudicate_claim(
                        report, key, probe, metric, statistic, budget, summaries
                    )

    report["relative_verdict"] = (
        "regression" if report["relative_regressions"] else
        "inconclusive_noise" if report["inconclusive_noise"] else
        "resolved_within_relative_budgets"
    )
    report["absolute_verdict"] = (
        "violation" if report["absolute_contract_violations"] else "within contracts"
    )
    code = (1 if report["relative_regressions"] else
            4 if report["absolute_contract_violations"] else
            5 if report["inconclusive_noise"] else 0)
    report["status"] = (
        "performance_pass" if code == 0 else "inconclusive" if code == 5 else "rejected"
    )
    report["performance_acceptance"] = code == 0
    return report, code


def _adjudicate_claim(report, case, probe, metric, statistic, budget, summaries):
    values = {slot: summaries[slot][metric][statistic] for slot in CAPTURE_ORDER}
    paired_ratios = {
        "A1_to_B1": _finite_ratio(values["B1"], values["A1"]),
        "A2_to_B2": _finite_ratio(values["B2"], values["A2"]),
    }
    budget_ratio = Fraction(str(budget))
    claim = dict(case=case, probe=probe, metric=metric, statistic=statistic,
                 budget=budget, values=values,
                 paired_ratios={pair: None if ratio is None else float(ratio)
                                for pair, ratio in paired_ratios.items()})
    nonfinite_pair = any(ratio is None for ratio in paired_ratios.values())
    over_budget = nonfinite_pair or any(
        ratio > budget_ratio for ratio in paired_ratios.values()
    )
    if over_budget:
        claim["reason"] = (
            "nonzero candidate over zero baseline" if nonfinite_pair else
            "at least one paired ratio exceeds budget"
        )
        report["relative_regressions"].append(claim)
        return

    forward_noise = _finite_ratio(values["A2"], values["A1"])
    reverse_noise = _finite_ratio(values["A1"], values["A2"])
    baseline_noise = (
        None if forward_noise is None or reverse_noise is None else
        max(forward_noise, reverse_noise) - 1
    )
    headrooms = {
        pair: (budget_ratio - ratio) / budget_ratio
        for pair, ratio in paired_ratios.items()
    }
    remaining_headroom = min(headrooms.values())
    claim["baseline_noise"] = None if baseline_noise is None else float(baseline_noise)
    claim["paired_remaining_headroom"] = {
        pair: float(headroom) for pair, headroom in headrooms.items()
    }
    claim["remaining_headroom"] = float(remaining_headroom)
    if baseline_noise is None or baseline_noise > remaining_headroom:
        claim["reason"] = (
            "baseline A/A includes nonzero over zero" if baseline_noise is None else
            "symmetric A/A noise exceeds conservative paired remaining headroom"
        )
        report["inconclusive_noise"].append(claim)
    else:
        report["resolved_relative_claims"].append(claim)
