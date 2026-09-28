"""Build the runner config for Road 1 Milestone 1B (Query Constitution Enforcement).

Encodes the multi-model policy in one place so it is easy to audit and tweak:

- REVIEW family (boundary reviewer and review turns): codex / gpt-5.6-sol / high
- IMPLEMENTER family (plan, implement, and repair turns): codex / gpt-5.6-sol / medium

Both families persist their own provider session (reuse_session) thanks to the
family-keyed thread store in the projector. Cross-provider handoffs (plan and
findings) travel through the consumer prompt overlays in
automation/consumer_prompts/, since a grok session cannot resume a codex thread.

Run from the workspace root:

    PYTHONPATH=automation/runner/src python automation/runner/config/build_road1_m1b.py

Then validate:

    PYTHONPATH=automation/runner/src python -m runner.facade.cli validate \
        automation/runner/config/road1-m1b-query-constitution.json

Edit MODEL ids below if your local codex/grok CLIs expect different strings.
"""
from __future__ import annotations

import json
from pathlib import Path

from runner.generation.scaffold_templates import scaffold_config, telegram_notification_policy
from runner.generation.scaffold_types import ScaffoldRequest

CONFIG_PATH = Path("automation/runner/config/road1-m1b-query-constitution.json")
SPEC_FILE = "cad/docs/road-1/road-1_milestone-1b.md"

# Explicit command paths: neither CLI is on PATH on this machine, and the
# adapters default to bare "codex"/"grok". Both model ids are confirmed against
# the local CLIs' model caches (codex slug gpt-5.6-sol, grok id grok-4.5).
CODEX_CMD = "C:/nvm4w/nodejs/codex.cmd"
GROK_CMD = "C:/Users/Esther/.grok/bin/grok.exe"

OVERSIGHT = {"provider": "codex", "command": CODEX_CMD, "model": "gpt-5.6-sol", "reasoning_effort": "high"}
IMPLEMENTER = {"provider": "codex", "command": CODEX_CMD, "model": "gpt-5.6-sol", "reasoning_effort": "medium"}
GROK = {"provider": "grok", "command": GROK_CMD, "model": "grok-4.5"}

# turn -> (continuity_family, model_policy, overlay assembly id)
TURNS = {
    "boundary_review":        ("oversight", OVERSIGHT, "turns/boundary_review_m1b"),
    "plan":                   ("implementer", IMPLEMENTER, "turns/plan_m1b"),
    "implement":              ("implementer", IMPLEMENTER, "turns/implement_m1b"),
    "review":                 ("oversight", OVERSIGHT, "turns/review_m1b"),
    "repair":                 ("implementer", IMPLEMENTER, "turns/repair_m1b"),
    "test_review":            ("oversight", OVERSIGHT, "turns/test_review_m1b"),
    "test_repair_implement":  ("implementer", IMPLEMENTER, "turns/test_repair_implement_m1b"),
    "code_quality_review":    ("oversight", OVERSIGHT, "turns/code_quality_review_m1b"),
    "code_quality_repair":    ("implementer", IMPLEMENTER, "turns/code_quality_repair_m1b"),
}

CONTEXT_FILES = [
    "docs/coding-guidelines/MENTALITY.md",
    "docs/coding-guidelines/arch_laws.md",
    "docs/coding-guidelines/composition_laws.md",
    "docs/coding-guidelines/domain_structure_laws.md",
    "docs/coding-guidelines/perf_laws.md",
    "docs/coding-guidelines/dx_laws.md",
    "cad/docs/worthy-foundations/ROAD.md",
    "cad/docs/worthy-foundations/ARCHITECTURE.md",
    "cad/docs/worthy-foundations/GLOSSARY.md",
    "cad/docs/worthy-foundations/BOUNDARIES.md",
    "cad/docs/worthy-foundations/NAMING.md",
    "cad/docs/road-1/road-1.md",
    SPEC_FILE,
    "crates/worth-query/docs/AI_README.md",
    "crates/worth-proof/README.md",
]

GREEN = "boundary-check and agent-context are green"

PHASES = [
    {
        "key": "residue-and-rename-ratchet",
        "title": "Milestone 1 Residue And Rename Ratchet",
        "scope": ["cad/workspaces/worth-contracts/", "cad/workspaces/worth-packs/", "tools/boundary-check/"],
        "acceptance": [
            "worth-schema-core has no top-level tests/ tree; the deep-import proof is relocated and the seed skeleton allowlist shrinks to match",
            "PackRegistration composes ContributionDescriptor instead of duplicating its fields",
            "a committed legacy-references snapshot rejects any new forge_/forge- reference under governed surfaces and may only shrink",
            GREEN,
        ],
        "qa_focus": "No redesign beyond the named cleanups; the rename ratchet governs only constitutional surfaces, not plans history or non-governed legacy code.",
    },
    {
        "key": "query-audience-facade-topology",
        "title": "Query Audience Facade Topology",
        "scope": ["crates/worth-query/", "crates/worth-query-decl/", "crates/worth-query-host/", "crates/worth-query-replay/", "cad/docs/worthy-foundations/NAMING.md"],
        "acceptance": [
            "worth-query-decl, worth-query-host, worth-query-replay exist as re-export-only facades over worth-query with doctest-bearing surfaces",
            "an entry-band consumer sees types identical to the engine facade through decl/host (no wrapper drift)",
            "a governed crate taking a direct worth-query dependency is denied with the audience facade named",
            "the audience matrix is recorded in NAMING.md as a framework-family amendment in this change",
            GREEN,
        ],
        "qa_focus": "Facades re-export only; no engine-internal split; no facade re-exports another facade; no speculative bridge nouns.",
    },
    {
        "key": "authority-sealing-law",
        "title": "Authority Sealing Law",
        "scope": ["tools/boundary-check/", "crates/worth-proof/"],
        "acceptance": [
            "a governed public item generic over AuthorityMarker/CapabilityMarker/AuthorityProves is rejected with the sealing law quoted and the concrete pattern shown",
            "worth-proof is recorded in machine config as a law substrate legal in every band and tier",
            "a forged local authority satisfies worth-proof generics but fails every governed ceremony signature",
            GREEN,
        ],
        "qa_focus": "worth-proof stays open; only the platform's vocabulary is sealed; no temporary generic authority bounds on governed surfaces.",
    },
    {
        "key": "band-guard-macro",
        "title": "Band Guard Macro",
        "scope": ["crates/worth-proof/"],
        "acceptance": [
            "worth_proof::band_guard! exists as a dependency-free const assertion over env!(CARGO_PKG_NAME) and is zero-cost",
            "expanding a guarded macro in a wrong-band fixture fails with the legal band list in the error text",
            "the adoption law (every public Query-facade macro embeds a band guard) is recorded",
            GREEN,
        ],
        "qa_focus": "The band list is passed as macro arguments, not encoded in worth-proof; the guard backstops macro-carried surfaces, it does not replace DAG rules.",
    },
    {
        "key": "source-level-import-law",
        "title": "Source-Level Import Law",
        "scope": ["tools/boundary-check/"],
        "acceptance": [
            "the AST pass denies worth_query-rooted source paths in governed crates outside the audience matrix",
            "Query types in governed public signatures and pub use of Query items are denied with distinct diagnostic codes",
            "a renamed-dependency engine import is still caught by the metadata pass (the two passes overlap, not gap)",
            GREEN,
        ],
        "qa_focus": "No full type resolution; the pass states what it cannot see; it runs only over governed crates and stays edit-loop fast.",
    },
    {
        "key": "surface-and-dag-ratchets",
        "title": "Surface And DAG Ratchets",
        "scope": ["tools/boundary-check/", "tools/agent-context/"],
        "acceptance": [
            "crate-dag and facade manifest snapshots exist as exact sets; additions and removals both require an in-change snapshot update",
            "a new governed dependency edge or a new pub use in a governed facade fails until its snapshot is regenerated, showing the widening as a reviewable diff",
            "agent-context renders its Facade exports lines from the facade snapshot rather than re-deriving them",
            GREEN,
        ],
        "qa_focus": "Exact-set equality, not subset; snapshots are writer-flag-gated and never hand-edited; only governed crates are snapshotted.",
    },
    {
        "key": "edit-time-enforcement",
        "title": "Edit-Time Enforcement And Prescriptive Diagnostics",
        "scope": ["scripts/", ".github/workflows/ci.yml", "tools/boundary-check/"],
        "acceptance": [
            "a single check-constitution entrypoint (boundary-check + agent-context check) is invoked identically by an edit-time hook, CI, and the terminal",
            "every Diagnostic carries a non-empty legal_home pointer, proven by a unit test over every diagnostic constructor",
            "a PostToolUse hook runs the entrypoint on governed edits and surfaces JSON diagnostics naming the legal home",
            GREEN,
        ],
        "qa_focus": "Hook, CI, and terminal share one entrypoint and one config; the hook stays within a stated time budget; pointers name a machine artifact first.",
    },
    {
        "key": "cert-corpus-birth",
        "title": "Cert Corpus Birth",
        "scope": ["cad/workspaces/worth-certification/", "cad/docs/worthy-foundations/NAMING.md"],
        "acceptance": [
            "worth-cert-adoption is born at its reserved name with a trybuild compile-fail corpus",
            "forged-authority, deep-import, replay-in-ordinary-band, band-guard-wrong-band, and generic-authority-bound specimens each fail as designed against stable diagnostic fragments",
            "a facade-pairing check rejects deletion of any specimen, proving the corpus is load-bearing",
            GREEN,
        ],
        "qa_focus": "The corpus proves denial, not features; no parity/scale/regression tree here (Milestone 5 owns those); nothing ordinary depends back on cert.",
    },
]


# M1B runs every configured turn in normal mode. Goal mode remains available
# only through an explicit operator custom turn prefixed with "goal".
REPAIR_TURNS = {"repair", "test_repair_implement", "code_quality_repair"}
GOAL_MODE_REPAIR = False


def role_binding(turn: str) -> dict:
    family, model_policy, assembly_id = TURNS[turn]
    model_policy = dict(model_policy)
    if GOAL_MODE_REPAIR and turn in REPAIR_TURNS:
        model_policy["goal_mode"] = True
    return {
        "role_id": "reviewer" if "review" in turn else "implementer",
        "model_policy": model_policy,
        "session_policy": {"continuity_family": family, "reuse_session": True},
        "prompt_template": {"assembly_id": assembly_id},
    }


def build_phase(index: int, phase: dict) -> dict:
    return {
        "id": index,
        "phase_key": phase["key"],
        "title": phase["title"],
        "owner": "implementer",
        "scope": phase["scope"],
        "acceptance": phase["acceptance"],
        "instructions": (
            f"Implement Milestone 1B phase '{phase['title']}' exactly as specified in "
            f"{SPEC_FILE}. Follow the phase's Warnings and Engineering decisions. Do not "
            f"reach into another phase's scope or into Milestone 3/4 semantics. Prove the "
            f"phase's adversarial tests and leave the constitution green."
        ),
        "qa_focus": phase["qa_focus"],
        "program_id": "standard_loop",
        "contract_template": {"asset_id": "contracts/m1b_simple"},
        "role_bindings": {turn: role_binding(turn) for turn in TURNS},
    }


def main() -> None:
    config = scaffold_config(ScaffoldRequest("milestone", "road1-m1b-query-constitution", str(Path.cwd()), SPEC_FILE))
    config["project"] = {
        "name": "Road 1 Milestone 1B: Query Constitution Enforcement",
        "cwd": str(Path.cwd()),
        "spec_file": SPEC_FILE,
        "context_files": CONTEXT_FILES,
    }
    config["prompt_library_policy"] = {
        "runner_asset_roots": ["automation/runner/prompts/assets"],
        "runner_assembly_roots": ["automation/runner/prompts/assemblies"],
        "consumer_asset_roots": ["automation/consumer_prompts/assets"],
        "consumer_assembly_roots": ["automation/consumer_prompts/assemblies"],
        "allow_consumer_prompts": True,
        "allow_direct_file_binding": False,
    }
    config["session_defaults"] = {**OVERSIGHT, "config": {}}
    notification_policy = telegram_notification_policy()
    # Routine faults stay silent while the configured recovery ladder still has
    # an automatic next step. Exhausted recovery becomes an operator_pause,
    # which is delivered through the blocker signal below.
    for signal_kind in ("crash", "invalid_outcome", "same_phase_loop_exceeded"):
        notification_policy["signals"][signal_kind]["enabled"] = False
    config["notification_policy"] = notification_policy
    config["runner_control"] = {"boundary_review_start_phase": 1}
    config["turn_templates"] = {turn: {"assembly_id": assembly} for turn, (_f, _m, assembly) in TURNS.items()}
    config["phases"] = [build_phase(i + 1, phase) for i, phase in enumerate(PHASES)]

    # Review escalation ladder (one config of the general escalation engine):
    # 3 review-family failures -> fresh session; next 3 -> escalate the repair
    # turns to codex gpt-5.6-sol high for the rest of the phase; next 3 -> pause
    # and page. Each fresh session re-arms the 3-window, giving the 3/3/3 cadence.
    config["loop_escalation"] = {
        "families": {
            "review_loop": {
                "turns": ["review", "test_review", "code_quality_review"],
                "threshold": 6,
                "action": "start_fresh_session",
            }
        }
    }
    config["escalation_policy"] = {
        "same_phase_loop_exceeded": {
            "stages": [
                {"action": "start_fresh_session"},
                {
                    "action": "override_model",
                    "turns": ["repair", "test_repair_implement", "code_quality_repair"],
                    "model_policy": {**OVERSIGHT, "goal_mode": GOAL_MODE_REPAIR},
                    "scope": "phase",
                },
            ],
            "on_exhausted": {"action": "notify_and_pause"},
        },
        "provider_crash": {"stages": [{"action": "start_fresh_session"}], "on_exhausted": "notify_and_pause"},
        "invalid_outcome": {"stages": [{"action": "start_fresh_session"}], "on_exhausted": "notify"},
        "no_edit_stall": {"stages": [{"action": "start_fresh_session"}], "on_exhausted": "notify"},
    }
    # Operator custom turn: reply to a blocker on Telegram with "<codex|grok>
    # <instructions>". The named (or default) model runs one turn with those
    # instructions, then the standard runner resumes. The per-phase cap bounds
    # how many times the ladder resets before it stays paged and paused.
    config["operator_custom_turn"] = {
        "aliases": {"codex": dict(IMPLEMENTER), "grok": dict(GROK)},
        "default_alias": "codex",
        "max_ladders_per_phase": 2,
    }

    CONFIG_PATH.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {CONFIG_PATH}")


if __name__ == "__main__":
    main()
