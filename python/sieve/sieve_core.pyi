from enum import Enum
from typing import overload

class SiftLevel(Enum):
    Low = 0
    Medium = 1
    High = 2

class SiftResult:
    @property
    def original(self) -> str: ...
    @property
    def sifted(self) -> str: ...
    @property
    def tokens_removed(self) -> int: ...
    @property
    def compression_ratio(self) -> float: ...

    def __repr__(self) -> str: ...

@overload
def sift(prompt: str) -> SiftResult: ...

@overload
def sift(prompt: str, level: SiftLevel) -> SiftResult: ...

