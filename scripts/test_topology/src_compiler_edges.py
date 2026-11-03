#!/usr/bin/env python3
"""Harvest method-call dependencies from compiler evidence into the source court overrides.

Usage:
    python scripts/test_topology/src_compiler_edges.py <cargo-check.json> <overrides.json> [--root .]

A call resolved by method name alone is unsound (std methods share names with test code), so
the classifier has no method edges. When compiled library code calls a method only a
`cfg(test)` module defines, `cargo check` reports E0599 at the call. This records
`caller module -> defining test module` under `compiler_method_edges` in the reviewed overrides
file, on the original (unapplied) tree, so the next plan classifies the caller correctly.
"""
import argparse
import json
from pathlib import Path
import re

from module_tree import read_tree
from src_families import families

LIB = 'workspaces/worth-query/crates/worth-query-execution/src/lib.rs'
METHOD = re.compile(r'no method named `(\w+)` found')


def module_of_file(tree, path):
    best = None
    for node in tree.walk():
        if node.declaration is not None and node.declaration.body is not None:
            continue
        if node.source == path and (best is None or len(node.names) > len(best)):
            best = node.names
    return best


def harvest(check_json, root):
    tree, found = families(root / LIB)
    family_nodes = [(f.names, node) for f in found for node in f.node.walk()]
    edges = {}
    for line in Path(check_json).read_text(encoding='utf-8').splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get('reason') != 'compiler-message' or message['message']['level'] != 'error':
            continue
        diagnostic = message['message']
        if (diagnostic.get('code') or {}).get('code') != 'E0599':
            continue
        match = METHOD.search(diagnostic['message'])
        span = next((s for s in diagnostic['spans'] if s['is_primary']), None)
        if match is None or span is None:
            continue
        workspace = root / 'workspaces/worth-query'
        caller = module_of_file(tree, (workspace / span['file_name']).resolve())
        if caller is None:
            continue
        pattern = re.compile(r'\bfn\s+' + match.group(1) + r'\b')
        definers = sorted({names for names, node in family_nodes if pattern.search(node.own_text())})
        for definer in definers:
            if caller[:len(definer)] != definer:
                edges.setdefault('::'.join(caller), set()).add('::'.join(definer))
    return {k: sorted(v) for k, v in edges.items()}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('check', type=Path)
    parser.add_argument('overrides', type=Path)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    overrides = json.loads(args.overrides.read_text(encoding='utf-8'))
    merged = {k: set(v) for k, v in overrides.get('compiler_method_edges', {}).items()}
    for caller, definers in harvest(args.check, args.root.resolve()).items():
        merged.setdefault(caller, set()).update(definers)
    overrides['compiler_method_edges'] = {k: sorted(v) for k, v in sorted(merged.items())}
    args.overrides.write_text(json.dumps(overrides, indent=2, sort_keys=True) + '\n', encoding='utf-8', newline='\n')
    print(json.dumps(overrides['compiler_method_edges'], indent=1))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
