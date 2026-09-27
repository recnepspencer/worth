#!/usr/bin/env python3
"""Apply the reviewed Worth token replacement matrix occurrence by occurrence.

This intentionally does not do broad string replacement. Some literal WORTH
tokens are valid constants, env vars, repository URLs, or fixture data. The
matrix is the authority for what changes and what stays put.
"""

from __future__ import annotations

import argparse
import csv
from dataclasses import dataclass
from pathlib import Path


DEFAULT_MATRIX = Path("plans/migration/worth-token-replacement-matrix.csv")


@dataclass(frozen=True)
class MatrixEdit:
    path: Path
    line_number: int
    token_occurrence_on_line: int
    current_token: str
    proposed_token: str


@dataclass(frozen=True)
class LineEdit:
    start: int
    end: int
    current_token: str
    proposed_token: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Apply non-keep replacements from the Worth token matrix."
    )
    parser.add_argument(
        "--matrix",
        type=Path,
        default=DEFAULT_MATRIX,
        help=f"Path to the reviewed matrix CSV. Defaults to {DEFAULT_MATRIX}.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate and report planned edits without writing files.",
    )
    return parser.parse_args()


def load_matrix_edits(matrix_path: Path) -> list[MatrixEdit]:
    edits_by_key: dict[tuple[str, int, int, str, str], MatrixEdit] = {}

    with matrix_path.open(newline="", encoding="utf-8") as matrix_file:
        reader = csv.DictReader(matrix_file)
        for row in reader:
            if row["confidence"] == "keep":
                continue
            if row["current_token"] == row["proposed_token"]:
                continue

            key = (
                row["path"],
                int(row["line"]),
                int(row["token_occurrence_on_line"]),
                row["current_token"],
                row["proposed_token"],
            )
            edits_by_key[key] = MatrixEdit(
                path=Path(row["path"]),
                line_number=int(row["line"]),
                token_occurrence_on_line=int(row["token_occurrence_on_line"]),
                current_token=row["current_token"],
                proposed_token=row["proposed_token"],
            )

    return sorted(
        edits_by_key.values(),
        key=lambda edit: (
            str(edit.path),
            edit.line_number,
            edit.token_occurrence_on_line,
            edit.current_token,
        ),
    )


def find_token_occurrence(line_body: str, token: str, occurrence: int) -> tuple[int, int]:
    if occurrence < 1:
        raise ValueError(f"Occurrence indexes are 1-based; got {occurrence}")

    matches: list[tuple[int, int]] = []
    search_from = 0
    while True:
        start = line_body.find(token, search_from)
        if start == -1:
            break
        end = start + len(token)
        matches.append((start, end))
        search_from = end

    if len(matches) == 1:
        return matches[0]
    if occurrence <= len(matches):
        return matches[occurrence - 1]

    raise ValueError(
        f"Could not find occurrence {occurrence} of token {token!r} "
        f"in line {line_body!r}"
    )


def split_line_ending(line: str) -> tuple[str, str]:
    if line.endswith("\r\n"):
        return line[:-2], "\r\n"
    if line.endswith("\n"):
        return line[:-1], "\n"
    return line, ""


def validate_non_overlapping(edits: list[LineEdit], path: Path, line_number: int) -> None:
    ordered = sorted(edits, key=lambda edit: edit.start)
    previous_end = -1
    for edit in ordered:
        if edit.start < previous_end:
            raise ValueError(
                f"Overlapping edits in {path}:{line_number} around "
                f"{edit.current_token!r}"
            )
        previous_end = edit.end


def apply_line_edits(line: str, edits: list[LineEdit]) -> str:
    body, ending = split_line_ending(line)
    updated = body
    for edit in sorted(edits, key=lambda item: item.start, reverse=True):
        actual = updated[edit.start : edit.end]
        if actual != edit.current_token:
            raise ValueError(
                f"Edit drift while applying {edit.current_token!r}; found {actual!r}"
            )
        updated = updated[: edit.start] + edit.proposed_token + updated[edit.end :]
    return updated + ending


def apply_edits(edits: list[MatrixEdit], dry_run: bool) -> tuple[int, int]:
    by_file: dict[Path, list[MatrixEdit]] = {}
    for edit in edits:
        by_file.setdefault(edit.path, []).append(edit)

    changed_files = 0
    applied_edits = 0

    for path, file_edits in sorted(by_file.items(), key=lambda item: str(item[0])):
        if not path.exists():
            raise FileNotFoundError(f"Matrix target does not exist: {path}")

        lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
        line_edits: dict[int, list[LineEdit]] = {}

        for edit in file_edits:
            if edit.line_number < 1 or edit.line_number > len(lines):
                raise ValueError(f"Matrix line is out of range: {path}:{edit.line_number}")

            line_body, _ = split_line_ending(lines[edit.line_number - 1])
            start, end = find_token_occurrence(
                line_body, edit.current_token, edit.token_occurrence_on_line
            )
            line_edits.setdefault(edit.line_number, []).append(
                LineEdit(
                    start=start,
                    end=end,
                    current_token=edit.current_token,
                    proposed_token=edit.proposed_token,
                )
            )

        for line_number, edits_for_line in line_edits.items():
            validate_non_overlapping(edits_for_line, path, line_number)
            lines[line_number - 1] = apply_line_edits(
                lines[line_number - 1], edits_for_line
            )
            applied_edits += len(edits_for_line)

        if not dry_run:
            path.write_text("".join(lines), encoding="utf-8")
        changed_files += 1

    return changed_files, applied_edits


def main() -> None:
    args = parse_args()
    edits = load_matrix_edits(args.matrix)
    changed_files, applied_edits = apply_edits(edits, args.dry_run)
    mode = "validated" if args.dry_run else "applied"
    print(f"{mode} {applied_edits} edits across {changed_files} files")


if __name__ == "__main__":
    main()
