"""Visibility restrictions as module scopes, and their restatement at a new path."""
from src_paths import resolve


def scope_of(visibility, module):
    """The module tuple a visibility restricts to, `()` for the crate, or 'pub'."""
    text = visibility.replace(' ', '')
    if text == 'pub':
        return 'pub'
    if text in ('', 'pub(self)'):
        return module
    if text == 'pub(super)':
        return module[:-1]
    if text == 'pub(crate)':
        return ()
    if text.startswith('pub(in') and text.endswith(')'):
        path = tuple(text[len('pub(in'):-1].split('::'))
        resolved = resolve(path, module, {}) if path[0] in ('super', 'self') else None
        if path[0] == 'crate':
            return path[1:]
        if resolved is not None:
            return resolved[0]
        return module
    raise ValueError('unknown visibility ' + visibility)


def stated(scope):
    if scope == 'pub':
        return 'pub'
    return 'pub(crate)' if scope == () else 'pub(in crate::' + '::'.join(scope) + ')'


def restated(visibility, module, parent):
    """(new visibility text, widened?) for a restriction written in `module`, moved below parent."""
    scope = scope_of(visibility, module)
    if scope == 'pub' or parent[:len(scope)] == scope:
        return stated(scope), False
    return stated(parent), True


def within(viewer, scope):
    """Whether module `viewer` lies inside `scope` ('pub' holds every module)."""
    if scope == 'pub':
        return True
    return viewer[:len(scope)] == scope
