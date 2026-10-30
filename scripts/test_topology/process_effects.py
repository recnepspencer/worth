"""Derive process/durability helper names from function bodies, never test names."""
import re
from functools import lru_cache

from rust_source import code, pairs, tokens

PROCESS_HELPERS = {'reexecute_isolated_test', 'run_isolated_test', 'spawn_child',
                   'run_child', 'spawn_process'}
DURABILITY_CALLS = {'try_admit_filesystem_media', 'FilesystemMediaAdmission',
                    'RelationalStoreHome', 'DirectoryHome', 'sync_all', 'sync_data'}
EFFECT = re.compile(r'\bCommand\s*:\s*:\s*new\b|\b(?:' + '|'.join(sorted(DURABILITY_CALLS)) + r')\b')
CALL = re.compile(r'\b([A-Za-z_][A-Za-z_0-9]*)\s*\(')


def direct_effect(value):
    return bool(EFFECT.search(value))


def helpers(tree):
    bodies = {}
    for node in tree.walk():
        text = node.own_text()
        stream = tokens(text)
        matching = pairs(stream)
        for i, token in enumerate(stream[:-1]):
            if token.value != 'fn':
                continue
            name = stream[i + 1].value
            opening = next((j for j in range(i + 2, len(stream)) if stream[j].value in ('{', ';')), None)
            if opening is not None and stream[opening].value == '{':
                body = text[stream[opening].end:stream[matching[opening]].start]
                bodies.setdefault(name, []).append(code(body))
    body_calls = {name: set().union(*(call_names(value) for value in values)) for name, values in bodies.items()}
    effected = set(PROCESS_HELPERS) | {name for name, values in bodies.items() if any(direct_effect(value) for value in values)}
    while True:
        found = {name for name, called in body_calls.items() if called & effected}
        expanded = effected | found
        if expanded == effected:
            return effected
        effected = expanded


def calls(value, names):
    return bool(call_names(value) & set(names))


@lru_cache(maxsize=512)
def call_names(value):
    return set(CALL.findall(value))
