from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Empty(_message.Message):
    __slots__ = []
    def __init__(self) -> None: ...

class InputRequest(_message.Message):
    __slots__ = ["x", "z"]
    X_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    x: float
    z: float
    def __init__(self, x: _Optional[float] = ..., z: _Optional[float] = ...) -> None: ...

class PlayerView(_message.Message):
    __slots__ = ["distance", "surrounding", "y"]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    SURROUNDING_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    distance: float
    surrounding: _containers.RepeatedCompositeFieldContainer[Terrain]
    y: float
    def __init__(self, surrounding: _Optional[_Iterable[_Union[Terrain, _Mapping]]] = ..., y: _Optional[float] = ..., distance: _Optional[float] = ...) -> None: ...

class Score(_message.Message):
    __slots__ = ["timings", "total"]
    TIMINGS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_FIELD_NUMBER: _ClassVar[int]
    timings: _containers.RepeatedScalarFieldContainer[int]
    total: int
    def __init__(self, timings: _Optional[_Iterable[int]] = ..., total: _Optional[int] = ...) -> None: ...

class Terrain(_message.Message):
    __slots__ = ["height", "kind"]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    height: float
    kind: int
    def __init__(self, height: _Optional[float] = ..., kind: _Optional[int] = ...) -> None: ...
