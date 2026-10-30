"""Prepare workspace-owned prerequisites before selected court processes run."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_cost'))
from cargo_capture import CargoRunner
from prebuild import prepare as prepare_workspace
from proof_prebuild import prepare as prepare_proof


class PrerequisiteFailure(RuntimeError):
    def __init__(self, workspace, cause):
        self.workspace = workspace
        self.cause = cause
        super().__init__(f'{workspace} prebuild failed: {cause}')


class GatePrebuild:
    def __init__(self, repository, output, environment, runner=None, prepare=None, proof=None):
        self.repository = Path(repository)
        self.output = Path(output)
        self.environment = environment
        self.runner = runner if runner is not None else CargoRunner(repository, environment)
        self.prepare_workspace = prepare if prepare is not None else prepare_workspace
        self.prepare_proof = proof if proof is not None else prepare_proof
        self.prepared = set()
        self.proof_prepared = False

    def prepare(self, group, selected_binaries):
        workspace = required_workspace(group, selected_binaries)
        proof_needed = workspace == 'root' and any(
            b['package'] == 'worth-court-proof' for b in selected_binaries)
        if workspace is None:
            return
        if workspace in self.prepared and not (proof_needed and not self.proof_prepared):
            return
        try:
            if workspace == 'root' and workspace in self.prepared:
                path = self.prepare_proof(self.repository, self.output / 'root/proof', self.runner)
                additions = {'WORTH_PREBUILT_PROOF': str(path)}
            else:
                options = {'include_proof': proof_needed} if workspace == 'root' else {}
                additions = self.prepare_workspace(self.repository, workspace,
                                                   self.output / workspace, self.runner, **options)
        except Exception as error:
            raise PrerequisiteFailure(workspace, error) from error
        self.environment.update(additions)
        self.prepared.add(workspace)
        self.proof_prepared |= proof_needed


def required_workspace(group, selected_binaries):
    if not selected_binaries:
        return None
    if group in ('root', 'query', 'bank'):
        return group
    return None


def refused_scale_suites(suites, census):
    """Scheduled scale entrypoints remain available outside census execution."""
    return [suite for suite in suites if census and suite.endswith('-scale')]
