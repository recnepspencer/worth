"""Identify opaque item macro invocations without pretending to expand Rust."""
from rust_source import pairs, tokens

BUILTINS = {'assert', 'assert_eq', 'assert_ne', 'debug_assert', 'debug_assert_eq',
            'debug_assert_ne', 'format', 'format_args', 'vec', 'println', 'eprintln',
            'print', 'eprint', 'panic', 'unreachable', 'unimplemented', 'todo', 'matches',
            'include', 'include_str', 'include_bytes', 'env', 'option_env', 'concat',
            'stringify', 'file', 'line', 'column', 'module_path', 'write', 'writeln',
            'thread_local', 'cfg', 'compile_error'}


def opaque_items(text):
    stream, result = tokens(text), set()
    matching = pairs(stream)
    i = 0
    while i < len(stream):
        token = stream[i]
        if token.kind == 'ident' and i + 1 < len(stream) and stream[i + 1].value == '!':
            if token.value not in BUILTINS:
                result.add(token.value)
        if token.value in ('{', '(', '['):
            i = matching[i] + 1
        else:
            i += 1
    return result
