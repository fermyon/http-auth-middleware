from typing import TypeVar, Generic, Union, Optional, Union, Protocol, Tuple, List, Any
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from ..types import Result, Ok, Err, Some
from ..imports import types

class IncomingHandler(Protocol):

    @abstractmethod
    def handle(self, request: types.IncomingRequest, response_out: types.ResponseOutparam) -> None:
        raise NotImplementedError


