"""Verify the preservation commit is a real merge of the qualified sources."""
import json
from pathlib import Path
import subprocess

def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()

head = git('rev-parse', 'HEAD')
parents = git('rev-list', '--parents', '-n', '1', head).split()[1:]
assert parents == ['25619ac5d1ae6092ab5a91bc4b7123be6042475c',
                   '14b16fa292e96ff0f5d96b55bd88c338a290cd87'], parents
assert git('status', '--short') == ''
assert git('branch', '--show-current') == 'codex/m3-p3-reopen'
source = 'cbfc0430ac01048febd893f214f766056417f287'
subprocess.run(['git', 'diff', '--quiet', source, head, '--', '.',
                ':(exclude)target/lane-evidence/**'], check=True)
for accepted in ['bd8d5b6af5', '9d04a90050', 'd0061f1327', '5e3baae5eb',
                 '1f9f39b138', '5692704eb7']:
    subprocess.run(['git', 'merge-base', '--is-ancestor', accepted, head], check=True)
message = git('log', '-1', '--format=%B')
assert message.endswith('Co-Authored-By: GPT-6.1 Sol <noreply@openai.com>')
assert 'IN PROGRESS' in message
record = {'head': head, 'parents': parents, 'status_short_empty': True,
          'reviewed_non_evidence_tree_matches': source,
          'accepted_funding_foundation_and_queue_ancestry_retained': True,
          'independent_review': 'REJECT constitutional closure; IN PROGRESS preservation only',
          'subject': git('log', '-1', '--format=%s')}
Path('target/lane-evidence/funding-merge-commit-identity.json').write_text(
    json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
