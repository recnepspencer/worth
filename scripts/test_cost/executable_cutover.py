"""Cut Store's named executable consumers over to a fallible prebuilt lookup."""
import argparse
from pathlib import Path
import re

from workspace import sources


CONSTANTS = ('OFFLINE_ROOT_OBSERVER', 'OFFLINE_RECOVERY_OBSERVER', 'FRESH_PROCESS_RECOVERY')
CALL = re.compile(r'(\bworth_store_test_support::(?:' + '|'.join(CONSTANTS) + r')\s*\.path\(\))')


def rewrite(text, provided=False):
    text, count = CALL.subn(r'\1.expect("runner must prebuild the independent executable")', text)
    if provided:
        text, extra = re.subn(r'(let path = executable\s*\.path\(\))',
                             r'\1.expect("runner must prebuild the independent executable")', text)
        count += extra
    return text, count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    manifest = Path('workspaces/worth-store/Cargo.toml')
    for _, path, text in sources(manifest):
        provided = path.as_posix().endswith('/support/support_binaries/declarations.rs')
        if '.expect("runner must prebuild' in text:
            continue
        changed, count = rewrite(text, provided)
        if count:
            print(f'{path}: {count}')
            if args.apply:
                io_path = Path('\\\\?\\' + str(path)) if __import__('os').name == 'nt' else path
                io_path.write_text(changed, encoding='utf-8')


if __name__ == '__main__':
    main()
