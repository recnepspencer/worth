"""Select existing consumer CI jobs from Cargo reach; this never qualifies code."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'test_cost'))
from cargo_reach import changed_paths, consumer_documents, local_package_dirs, workspace_hits
from query_gate_cost import DOC_SUITES
from query_gate_courts import FIXTURE
from workspace import WORKSPACES


def full(reason):
    return {'consumer_docs': True, 'checkpoint_recovery': True, 'reason': reason}


def select(repo, changed, discover=local_package_dirs):
    documents = consumer_documents(repo) | {'README.md'}
    if any(path.startswith('.github/') or
           (path.startswith('scripts/') and not path.startswith('scripts/test_cost/slots/')) or
           path.startswith('tools/boundary-check/config/consumer-docs') for path in changed):
        return full('qualification workflow, command or configured inventory changed')
    docs = any(path in documents for path in changed)
    candidates = [path for path in changed if path not in documents and
                  not path.startswith(('plans/', 'docs/', 'scripts/test_cost/slots/'))]
    result = {'consumer_docs': docs, 'checkpoint_recovery': False,
              'reason': 'configured documents and Cargo package/input reach'}
    if not candidates:
        return result
    try:
        for owner in dict.fromkeys(DOC_SUITES.values()):
            manifest = WORKSPACES[owner]
            if workspace_hits(candidates, manifest, discover(repo, manifest, offline=False)):
                result['consumer_docs'] = True
        if workspace_hits(candidates, FIXTURE, discover(repo, FIXTURE, offline=False)):
            result['checkpoint_recovery'] = True
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        return full('Cargo graph unavailable; retain both jobs: ' + str(error))
    return result


def selection(repo, event_name, event, discover=local_package_dirs):
    if not isinstance(event, dict):
        return full('event is not an object; retain full qualification')
    if event_name == 'workflow_dispatch':
        inputs = event.get('inputs') or {}
        if not isinstance(inputs, dict):
            return full('manual inputs are not an object; retain full qualification')
        scope = inputs.get('scope', 'all')
        if scope not in ('all', 'consumer-docs', 'checkpoint-only'):
            return full('unknown manual scope; retain full qualification')
        return {'consumer_docs': scope != 'checkpoint-only',
                'checkpoint_recovery': scope != 'consumer-docs', 'reason': 'explicit manual ' + scope}
    try:
        if event_name == 'pull_request':
            pull = event['pull_request']
            base, head = pull['base']['sha'], pull['head']['sha']
        elif event_name == 'push':
            base, head = event['before'], event['after']
        else:
            return full('unknown event; retain full qualification')
        changed = changed_paths(repo, base, head, merge_base=event_name == 'pull_request')
        result = select(repo, changed, discover)
        return {**result, 'changed_files': len(changed)}
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        return full('comparison or configured inventory unavailable; retain both jobs: ' + str(error))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--event-name', default=os.environ.get('GITHUB_EVENT_NAME', ''))
    parser.add_argument('--event-path', type=Path, default=os.environ.get('GITHUB_EVENT_PATH'))
    parser.add_argument('--output', type=Path, default=os.environ.get('GITHUB_OUTPUT'))
    args = parser.parse_args()
    try:
        event = json.loads(args.event_path.read_text(encoding='utf-8')) if args.event_path else {}
        result = selection(args.repository, args.event_name, event)
    except (OSError, ValueError, TypeError) as error:
        result = full('event unavailable; retain both jobs: ' + str(error))
    print(json.dumps(result, indent=2))
    if args.output:
        with args.output.open('a', encoding='utf-8') as stream:
            for name in ('consumer_docs', 'checkpoint_recovery'):
                stream.write(f'{name}={str(result[name]).lower()}\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
