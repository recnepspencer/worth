"""Planner handoff contracts use controlled syntax fixtures without external files."""
import ast
import unittest

from landing_plan import planner_hook_violations


PLANNER = '''import subprocess
import sys
from pathlib import Path

def validate_landing_plan(repo):
    checker = Path(repo).resolve() / "scripts/test_cost/landing_plan.py"
    observed = subprocess.run([sys.executable, "-I", str(checker),
                               "--repository", str(Path(repo).resolve()),
                               "--external", str(Path(__file__).resolve()),
                               "--check-only"],
                              capture_output=True, text=True)
    if observed.returncode:
        raise SystemExit("landing validation failed:" + observed.stdout + observed.stderr)

def main():
    repo, base = sys.argv[1], sys.argv[2]
    head = sys.argv[3] if len(sys.argv) > 3 else "HEAD"
    validate_landing_plan(repo)
    changed = run(["git", "diff", base, head], repo)
    local_package_dirs(repo, "Cargo.toml")
    print(changed)
'''


class LandingPlanHookTests(unittest.TestCase):
    def test_hook_uses_repository_checker_without_metadata_or_temporary_modes(self):
        self.assertEqual(planner_hook_violations(ast.parse(PLANNER)), [])
        for old, new in [('str(checker)', '"cargo"'),
                         ('"--check-only"]', '"--check-only", "--unmeasured-mode"]'),
                         ('"--check-only"', '"--emit-output"'),
                         ('Path(repo).resolve() /', 'Path("elsewhere").resolve() /')]:
            with self.subTest(change=new):
                self.assertTrue(planner_hook_violations(ast.parse(PLANNER.replace(old,new))))

    def test_hook_failure_refuses_plan(self):
        for replacement in ['if False:', 'if not observed.returncode:']:
            self.assertTrue(planner_hook_violations(ast.parse(PLANNER.replace(
                'if observed.returncode:',replacement))))
        self.assertTrue(planner_hook_violations(ast.parse(PLANNER.replace(
            'raise SystemExit(', 'print('))))

    def test_main_validates_before_git_or_any_cargo_metadata_process(self):
        marker = '    validate_landing_plan(repo)\n'
        for prior in ['    local_package_dirs(repo, "Cargo.toml")\n',
                      '    changed = run(["git", "diff"], repo)\n',
                      '    print("plan")\n']:
            with self.subTest(prior=prior):
                errors = planner_hook_violations(ast.parse(PLANNER.replace(marker,prior+marker)))
                self.assertTrue(any('precede' in row for row in errors))

    def test_missing_or_duplicate_validation_never_passes(self):
        marker = '    validate_landing_plan(repo)\n'
        for replacement in ['',marker+marker]:
            self.assertTrue(planner_hook_violations(ast.parse(PLANNER.replace(marker,replacement))))
        self.assertTrue(planner_hook_violations(ast.parse('def main():\n pass\n')))


if __name__ == '__main__':
    unittest.main()
