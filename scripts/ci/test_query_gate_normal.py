"""Pure committed-Git source planning; no Cargo, compiler or runtime execution."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from query_gate_normal import compile_plan, source_state


class NormalSourceTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.git('init', '-q'); self.git('config', 'user.name', 'normal court')
        self.git('config', 'user.email', 'normal@example.invalid')
        self.write('.gitignore', '__pycache__/\n')
        names = ['worth-execution', 'worth-court-execution']
        self.write('Cargo.toml', '[workspace]\nmembers=' + json.dumps(['crates/'+n for n in names]))
        for name in names:
            self.write('crates/'+name+'/Cargo.toml', '[package]\nname="'+name+'"\nversion="0.1.0"\n')
            self.write('crates/'+name+'/src/lib.rs', '// owner\n')
        self.commit(); self.base = self.git('rev-parse', 'HEAD')
        self.git('update-ref', 'refs/remotes/origin/master', self.base)

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, stderr=subprocess.STDOUT).decode().strip()

    def write(self, path, value):
        p=self.root/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(value)

    def commit(self):
        self.git('add', '.');self.git('commit', '-qm', 'source fixture')

    def test_owner_edit_derives_its_real_court_and_all_nonscale_categories(self):
        self.write('crates/worth-execution/src/lib.rs', '// actual edit\n');self.commit()
        plan=compile_plan(self.root,self.base)
        self.assertEqual(plan['packages'],['worth-court-execution','worth-execution'])
        self.assertEqual(plan['required_courts'],['worth-court-execution'])
        self.assertIn('execution',plan['suites']);self.assertIn('root-cost',plan['suites'])
        self.assertNotIn('root-scale',plan['suites']);self.assertNotIn('relational',plan['suites'])

    def test_court_only_edit_does_not_select_production_units(self):
        self.write('crates/worth-court-execution/src/lib.rs','// court edit\n');self.commit()
        self.assertEqual(compile_plan(self.root,self.base)['packages'],['worth-court-execution'])

    def test_source_only_adoption_is_not_an_empty_native_pass(self):
        self.write('scripts/ci/query_gate_normal.py','# tooling owner\n');self.commit()
        plan=compile_plan(self.root,self.base)
        self.assertEqual((plan['mode'],plan['packages'],plan['suites']),('source-only',[],[]))

    def test_nested_manifest_unknown_config_assets_and_docs_refuse(self):
        for path in ['config.toml','crates/worth-execution/asset.png','docs/unowned.md']:
            self.write(path,'unowned\n');self.commit()
            with self.assertRaisesRegex(ValueError,'unsupported normal input'):
                compile_plan(self.root,self.base)
            self.git('rm',path);self.commit()
        self.write('crates/worth-execution/fixture/Cargo.toml','[workspace]\n')
        self.write('crates/worth-execution/fixture/src/lib.rs','// foreign\n');self.commit()
        with self.assertRaises(ValueError):compile_plan(self.root,self.base)

    def test_rename_and_deletion_keep_the_old_owner_obligation(self):
        self.git('mv','crates/worth-execution/src/lib.rs','crates/worth-execution/src/renamed.rs');self.commit()
        plan=compile_plan(self.root,self.base)
        self.assertEqual(len(plan['paths']),2)
        self.assertIn('worth-court-execution',plan['packages'])

    def test_dirty_untracked_hidden_and_stale_base_refuse(self):
        self.write('untracked','unbound')
        with self.assertRaises(ValueError):source_state(self.root)
        (self.root/'untracked').unlink()
        path='crates/worth-execution/src/lib.rs'
        self.git('update-index','--assume-unchanged',path)
        with self.assertRaises(ValueError):source_state(self.root)
        self.git('update-index','--no-assume-unchanged',path)
        self.write(path,'// committed\n');self.commit()
        self.git('update-ref','refs/remotes/origin/master','HEAD')
        with self.assertRaisesRegex(ValueError,'fetched master'):compile_plan(self.root,self.base)

    def test_executable_and_symlink_input_modes_refuse(self):
        path = 'crates/worth-execution/src/lib.rs'
        self.write(path, '// mode change\n')
        self.git('add', path); self.git('update-index', '--chmod=+x', path)
        self.git('commit', '-qm', 'executable source')
        # Mirror the committed mode in the working tree rather than relying on core.filemode.
        (self.root/path).chmod(0o755)
        with self.assertRaisesRegex(ValueError, 'input mode'):
            compile_plan(self.root, self.base)
        self.git('rm', path)
        (self.root/path).parent.mkdir(parents=True, exist_ok=True)
        (self.root/path).symlink_to('another.rs')
        self.commit()
        with self.assertRaisesRegex(ValueError, 'input mode'):
            compile_plan(self.root, self.base)

    def test_missing_forward_owner_and_guide_only_refuse(self):
        self.write('docs/normal-gate.md', 'guide without provider\n'); self.commit()
        with self.assertRaisesRegex(ValueError, 'actual provider'):
            compile_plan(self.root, self.base)
        self.git('rm', 'docs/normal-gate.md'); self.commit()
        self.write('crates/worth-court-execution/src/lib.rs', '// changed court\n'); self.commit()
        self.git('rm', 'crates/worth-court-execution/Cargo.toml'); self.commit()
        with self.assertRaises(ValueError):
            compile_plan(self.root, self.base)
