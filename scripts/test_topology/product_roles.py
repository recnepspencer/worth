"""Classify resolved workspace packages before A2's product-frontier calculation."""
from pathlib import Path


def classify(metadata, established_products=()):
    members = set(metadata['workspace_members'])
    packages = {p['id']: p for p in metadata['packages']}
    incoming = {identity: [] for identity in members}
    for node in metadata['resolve']['nodes']:
        for edge in node['deps']:
            if edge['pkg'] in incoming:
                incoming[edge['pkg']].append((node['id'], edge['dep_kinds']))
    roles = {}
    for identity in sorted(members):
        p = packages[identity]
        name, parts = p['name'], Path(p['manifest_path']).parts
        if '-court-' in name:
            role = 'court'
        elif 'fixtures' in parts:
            role = 'fixture'
        elif 'tools' in parts or (name.endswith('-runner') and not any(
                'lib' in t['kind'] or 'rlib' in t['kind'] for t in p.get('targets', []))):
            role = 'tool'
        elif any(segment in name for segment in ('-testkit', '-test-support', '-certification', '-formal-models')) or name.endswith('-harness'):
            role = 'testkit'
        else:
            role = 'product'
        roles[identity] = role
    while True:
        discovered = [identity for identity in sorted(members) if roles[identity] == 'product' and packages[identity]['name'] not in established_products
                      and incoming[identity] and all(
                          roles.get(caller) in ('court', 'testkit', 'fixture', 'tool')
                          or all(k['kind'] == 'dev' for k in kinds)
                          for caller, kinds in incoming[identity])]
        if not discovered:
            return roles
        for identity in discovered:
            roles[identity] = 'testkit'
