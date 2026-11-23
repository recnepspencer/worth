"""Defining path of a mangled Rust symbol (v0 `_R...` or legacy `_ZN...E`).

Only what the frame budget lane needs: the crate and module path where a
function is defined, with instantiation details dropped. A function generic
over an owned type is defined by its own crate, so `core::ptr::drop_in_place`
over a WORTH type stays `core`.
"""
from __future__ import annotations

import re

BASE62 = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
BASIC_TYPES = set("abcdefhijlmnostuvxyzp")


class DemangleError(ValueError):
    """Not a symbol this parser understands."""


class _V0:
    def __init__(self, text: str) -> None:
        self.text = text
        self.at = 0
        self.depth = 0

    def peek(self) -> str:
        return self.text[self.at] if self.at < len(self.text) else ""

    def take(self) -> str:
        if self.at >= len(self.text):
            raise DemangleError("truncated")
        char = self.text[self.at]
        self.at += 1
        return char

    def eat(self, char: str) -> bool:
        if self.peek() == char:
            self.at += 1
            return True
        return False

    def base62(self) -> int:
        if self.eat("_"):
            return 0
        value = 0
        while not self.eat("_"):
            char = self.take()
            if char not in BASE62:
                raise DemangleError("bad base-62 digit")
            value = value * 62 + BASE62.index(char)
        return value + 1

    def opt_base62(self, tag: str) -> None:
        if self.eat(tag):
            self.base62()

    def decimal(self) -> int:
        start = self.at
        while self.peek().isdigit():
            self.at += 1
        if start == self.at:
            raise DemangleError("expected decimal")
        return int(self.text[start : self.at])

    def ident(self) -> str:
        self.opt_base62("s")
        punycode = self.eat("u")
        length = self.decimal()
        self.eat("_")
        name = self.text[self.at : self.at + length]
        self.at += length
        return ("punycode_" + name) if punycode else name

    def backref(self, parse):
        target = self.base62() + 2  # offsets count from after "_R"
        saved, self.at = self.at, target
        self.depth += 1
        if self.depth > 64:
            raise DemangleError("backref depth")
        try:
            return parse()
        finally:
            self.depth -= 1
            self.at = saved

    def path(self) -> list[str]:
        """Segments of the defining path; impl paths name their module."""
        tag = self.take()
        if tag == "C":
            return [self.ident()]
        if tag == "N":
            namespace = self.take()
            parent = self.path()
            name = self.ident()
            if namespace.islower() or not name:
                return parent + ([name] if name else [])
            return parent + [f"{{{'closure' if namespace == 'C' else namespace}}}"] + ([name] if name else [])
        if tag in "MX":
            self.opt_base62("s")
            parent = self.path()
            self.type_()
            if tag == "X":
                self.path()
            return parent + ["{impl}"]
        if tag == "Y":
            self.type_()
            return self.path()
        if tag == "I":
            parent = self.path()
            while not self.eat("E"):
                self.generic_arg()
            return parent
        if tag == "B":
            return self.backref(self.path)
        raise DemangleError(f"unknown path tag {tag!r}")

    def generic_arg(self) -> None:
        if self.eat("L"):
            self.base62()
        elif self.eat("K"):
            self.const()
        else:
            self.type_()

    def const(self) -> None:
        if self.eat("p"):
            return
        if self.eat("B"):
            self.backref(self.const)
            return
        self.type_()
        self.eat("n")
        while not self.eat("_"):
            if self.take() not in "0123456789abcdef":
                raise DemangleError("bad const")

    def binder(self) -> None:
        self.opt_base62("G")

    def type_(self) -> None:
        tag = self.peek()
        if tag in BASIC_TYPES:
            self.at += 1
            return
        if tag in "CNMXYI":
            self.path()
            return
        self.at += 1
        if tag == "B":
            self.backref(self.type_)
        elif tag == "A":
            self.type_()
            self.const()
        elif tag in "SPO":
            self.type_()
        elif tag in "RQ":
            self.opt_base62("L")
            self.type_()
        elif tag == "T":
            while not self.eat("E"):
                self.type_()
        elif tag == "F":
            self.binder()
            self.eat("U")
            if self.eat("K"):
                if not self.eat("C"):
                    self.ident()
            while not self.eat("E"):
                self.type_()
            self.type_()
        elif tag == "D":
            self.binder()
            while not self.eat("E"):
                self.path()
                while self.eat("p"):
                    self.ident()
                    self.type_()
            self.opt_base62("L")
        else:
            raise DemangleError(f"unknown type tag {tag!r}")


LEGACY_ESCAPES = {"$LT$": "<", "$GT$": ">", "$u20$": " ", "$RF$": "&", "$C$": ",", "$u7b$": "{", "$u7d$": "}"}


def _legacy(symbol: str) -> list[str]:
    body = symbol[3:]
    segments = []
    at = 0
    while at < len(body) and body[at] != "E":
        match = re.match(r"\d+", body[at:])
        if not match:
            raise DemangleError("bad legacy segment")
        length = int(match.group())
        at += len(match.group())
        segment = body[at : at + length]
        at += length
        for escape, char in LEGACY_ESCAPES.items():
            segment = segment.replace(escape, char)
        segments.append(segment.replace("..", "::"))
    if segments and re.fullmatch(r"h[0-9a-f]{16}", segments[-1]):
        segments.pop()
    if segments and segments[0].startswith("<"):
        # `<Type as Trait>::f`: the impl lives beside its self type.
        inner = segments[0][1:].split(" as ", 1)[0].lstrip("&")
        segments = inner.split("::") + ["{impl}"] + segments[1:]
    return segments


def defining_path(symbol: str) -> str | None:
    """`crate::module::item` for a mangled symbol, or None when not mangled."""
    name = symbol.lstrip("_")
    try:
        if name.startswith("ZN"):
            return "::".join(_legacy("_" + name))
        if name.startswith("R"):
            parser = _V0(name[1:])
            parser.text = "_R" + name[1:]
            parser.at = 2
            if parser.peek().isdigit():
                parser.decimal()
            return "::".join(parser.path())
    except (DemangleError, IndexError, ValueError):
        return None
    return None
