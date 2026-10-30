"""Production compiler graphs cannot activate the repository's test feature vocabulary."""
from pathlib import Path
import tomllib

from workspace import members


def vocabulary(workspace_manifest):
    manifest = Path(workspace_manifest).resolve()
    roots = {manifest}
    repository_manifest = manifest.parent.parent.parent / 'Cargo.toml'
    if manifest.parent.parent.name == 'workspaces' and repository_manifest.is_file():
        roots.add(repository_manifest)
    names = set()
    for root in roots:
        for package in members(root):
            data = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
            names.update(data.get('features', {}))
    return names


def is_test_feature(feature, declared):
    return ('certification' in feature or 'for-test' in feature or 'for_test' in feature or
            feature == 'test-support' or feature.startswith('test-') or feature.endswith('-for-tests') or
            feature in declared and feature.endswith('-fixtures'))


def violations(workspace_manifest, metadata):
    declared = vocabulary(workspace_manifest)
    return [(node['id'], feature) for node in metadata['resolve']['nodes']
            for feature in node['features'] if is_test_feature(feature, declared)]
