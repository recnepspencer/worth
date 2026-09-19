from __future__ import annotations

import re
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
MANIFEST = ROOT / "workspaces/worth-ui/contracts/milestone-3.16-docs.json"
EXPECTED_CONTINUING_DOCUMENTS = [
    "workspaces/worth-ui/docs/application-lifecycle.md",
    "workspaces/worth-ui/docs/authored-composition.md",
    "workspaces/worth-ui/docs/hot-rebind.md",
    "workspaces/worth-ui/docs/interaction-and-intents.md",
    "workspaces/worth-ui/docs/runtime-services.md",
    "workspaces/worth-ui/docs/runtime-subsystems.md",
    "workspaces/worth-ui/docs/text-platform.md",
    "workspaces/worth-ui/docs/native-host-platform.md",
    "workspaces/worth-ui/docs/inspection.md",
    "workspaces/worth-ui/docs/visual-inspection.md",
    "workspaces/worth-ui/docs/appearance-and-themes.md",
    "workspaces/worth-ui/AI_README.md",
    "_docs/worth-ui/worth_ui_roadmap.md",
    "_docs/worth-ui/milestone-3.16.md",
]
EXPECTED_PLANNED_DOCUMENTS: list[str] = []


def validate(root: Path, manifest: Path | None = None) -> None:
    if manifest is None:
        documents = sorted((root / "_docs/worth-ui").glob("**/*.md"))
        documents += sorted((root / "workspaces/worth-ui").glob("**/*.md"))
    else:
        contract = json.loads(manifest.read_text(encoding="utf-8"))
        if set(contract) != {"continuing_documents", "planned_documents"}:
            raise ValueError("documentation manifest keys must remain exact")
        if contract["continuing_documents"] != EXPECTED_CONTINUING_DOCUMENTS:
            raise ValueError("continuing Worth UI document set must remain exact")
        if contract["planned_documents"] != EXPECTED_PLANNED_DOCUMENTS:
            raise ValueError("planned Worth UI document set must remain exact")
        documents = [root / path for path in contract["continuing_documents"]]
    if not documents:
        raise ValueError("no continuing Worth UI documentation discovered")
    for document in documents:
        for target in LINK.findall(document.read_text(encoding="utf-8")):
            target = target.split("#", 1)[0]
            if not target or target.startswith(("http://", "https://", "mailto:")):
                continue
            resolved = (document.parent / target).resolve()
            if not resolved.exists():
                raise ValueError(f"{document.relative_to(root)}: broken local link {target}")


def main() -> int:
    try:
        validate(ROOT, MANIFEST)
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"Worth UI documentation/link gate failed: {error}", file=sys.stderr)
        return 1
    print("Worth UI documentation/link gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
