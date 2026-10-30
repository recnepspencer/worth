"""Declare the compiler proof stage before grouping fixture build graphs."""
from copy import deepcopy
from enum import Enum
import json
from pathlib import Path

from compiler_audiences import group, source_identity


class CompilerStage(Enum):
    CHECK = 'check'
    CODEGEN = 'codegen'

    @property
    def cargo_command(self):
        return {CompilerStage.CHECK: 'check', CompilerStage.CODEGEN: 'build'}[self]


def stage(environment):
    return CompilerStage(environment.get('proof_stage', CompilerStage.CHECK.value))


def partition(repository, workspace, environments, declarations=None):
    """Keep each original audience, splitting only explicitly declared stages."""
    repository = Path(repository).resolve()
    if declarations is None:
        declarations = json.loads(Path(__file__).with_name('compiler_stages.json').read_text())
    if declarations.get('schema') != 1:
        raise ValueError('unsupported compiler proof-stage declaration')
    declared = {}
    for declaration in declarations.get('workspaces', {}).get(workspace, []):
        path = (repository / declaration['source']).resolve()
        if not path.is_relative_to(repository):
            raise ValueError('compiler proof-stage source escapes repository')
        key = source_identity(path)
        if key in declared:
            raise ValueError(f'duplicate compiler proof-stage declaration: {path}')
        declared[key] = CompilerStage(declaration['stage'])
    consumed, result = set(), {}
    for name, environment in environments.items():
        records = {member: [] for member in CompilerStage}
        for package in environment['packages']:
            selected = {member: [] for member in CompilerStage}
            for source in package['sources']:
                key = source_identity(source)
                member = declared.get(key, CompilerStage.CHECK)
                if key in declared:
                    consumed.add(key)
                selected[member].append(source)
            for member, sources in selected.items():
                if sources:
                    records[member].append({
                        'dependencies': package['dependencies'],
                        'target_dependencies': package['target_dependencies'],
                        'audience': ', '.join(package['audiences']), 'sources': sources})
        for member, packages in records.items():
            if member is CompilerStage.CODEGEN and not packages:
                continue
            identity = name if member is CompilerStage.CHECK else name + '-codegen'
            if identity in result:
                raise ValueError(f'compiler stage environment collision: {identity}')
            result[identity] = dict(deepcopy(environment), identity=identity,
                                    environment_class=environment.get('environment_class', name),
                                    proof_stage=member.value, packages=group(packages))
    missing = set(declared) - consumed
    if missing:
        raise ValueError(f'compiler proof-stage declarations have no fixture: {sorted(missing)}')
    return result
