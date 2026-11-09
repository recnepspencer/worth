"""Typed refusals at the executable-placement boundary."""


class UnsafeLayout(ValueError):
    pass


class LoaderBoundsUnavailable(UnsafeLayout):
    pass
