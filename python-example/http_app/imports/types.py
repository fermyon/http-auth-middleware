from typing import TypeVar, Generic, Union, Optional, Union, Protocol, Tuple, List, Any
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from ..types import Result, Ok, Err, Some
import componentize_py_runtime
from ..imports import poll
from ..imports import streams


@dataclass
class MethodGet:
    pass


@dataclass
class MethodHead:
    pass


@dataclass
class MethodPost:
    pass


@dataclass
class MethodPut:
    pass


@dataclass
class MethodDelete:
    pass


@dataclass
class MethodConnect:
    pass


@dataclass
class MethodOptions:
    pass


@dataclass
class MethodTrace:
    pass


@dataclass
class MethodPatch:
    pass


@dataclass
class MethodOther:
    value: str


Method = Union[MethodGet, MethodHead, MethodPost, MethodPut, MethodDelete, MethodConnect, MethodOptions, MethodTrace, MethodPatch, MethodOther]


@dataclass
class SchemeHttp:
    pass


@dataclass
class SchemeHttps:
    pass


@dataclass
class SchemeOther:
    value: str


Scheme = Union[SchemeHttp, SchemeHttps, SchemeOther]


@dataclass
class ErrorInvalidUrl:
    value: str


@dataclass
class ErrorTimeoutError:
    value: str


@dataclass
class ErrorProtocolError:
    value: str


@dataclass
class ErrorUnexpectedError:
    value: str


Error = Union[ErrorInvalidUrl, ErrorTimeoutError, ErrorProtocolError, ErrorUnexpectedError]

class Fields:
    
    def __init__(self, entries: List[Tuple[str, bytes]]):
        tmp = componentize_py_runtime.call_import(33, [entries], 1)[0]
        (_, func, args, _) = tmp.finalizer.detach()
        self.handle = tmp.handle
        self.finalizer = weakref.finalize(self, func, args[0], args[1])

    def get(self, name: str) -> List[bytes]:
        result = componentize_py_runtime.call_import(34, [self, name], 1)
        return result[0]

    def set(self, name: str, value: List[bytes]) -> None:
        result = componentize_py_runtime.call_import(35, [self, name, value], 0)
        return

    def delete(self, name: str) -> None:
        result = componentize_py_runtime.call_import(36, [self, name], 0)
        return

    def append(self, name: str, value: bytes) -> None:
        result = componentize_py_runtime.call_import(37, [self, name, value], 0)
        return

    def entries(self) -> List[Tuple[str, bytes]]:
        result = componentize_py_runtime.call_import(38, [self], 1)
        return result[0]

    def clone(self) -> Fields:
        result = componentize_py_runtime.call_import(39, [self], 1)
        return result[0]

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class IncomingRequest:
    
    def method(self) -> Method:
        result = componentize_py_runtime.call_import(40, [self], 1)
        return result[0]

    def path_with_query(self) -> Optional[str]:
        result = componentize_py_runtime.call_import(41, [self], 1)
        return result[0]

    def scheme(self) -> Optional[Scheme]:
        result = componentize_py_runtime.call_import(42, [self], 1)
        return result[0]

    def authority(self) -> Optional[str]:
        result = componentize_py_runtime.call_import(43, [self], 1)
        return result[0]

    def headers(self) -> Fields:
        result = componentize_py_runtime.call_import(44, [self], 1)
        return result[0]

    def consume(self) -> IncomingBody:
        result = componentize_py_runtime.call_import(45, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class OutgoingRequest:
    
    def __init__(self, method: Method, path_with_query: Optional[str], scheme: Optional[Scheme], authority: Optional[str], headers: Fields):
        tmp = componentize_py_runtime.call_import(46, [method, path_with_query, scheme, authority, headers], 1)[0]
        (_, func, args, _) = tmp.finalizer.detach()
        self.handle = tmp.handle
        self.finalizer = weakref.finalize(self, func, args[0], args[1])

    def write(self) -> OutgoingBody:
        result = componentize_py_runtime.call_import(47, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


@dataclass
class RequestOptions:
    connect_timeout_ms: Optional[int]
    first_byte_timeout_ms: Optional[int]
    between_bytes_timeout_ms: Optional[int]

class ResponseOutparam:
    
    @staticmethod
    def set(param: ResponseOutparam, response: Result[OutgoingResponse, Error]) -> None:
        result = componentize_py_runtime.call_import(48, [param, response], 0)
        return

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class IncomingResponse:
    
    def status(self) -> int:
        result = componentize_py_runtime.call_import(49, [self], 1)
        return result[0]

    def headers(self) -> Fields:
        result = componentize_py_runtime.call_import(50, [self], 1)
        return result[0]

    def consume(self) -> IncomingBody:
        result = componentize_py_runtime.call_import(51, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class IncomingBody:
    
    def stream(self) -> streams.InputStream:
        result = componentize_py_runtime.call_import(52, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    @staticmethod
    def finish(this: IncomingBody) -> FutureTrailers:
        result = componentize_py_runtime.call_import(53, [this], 1)
        return result[0]

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class FutureTrailers:
    
    def subscribe(self) -> poll.Pollable:
        """
        Pollable that resolves when the body has been fully read, and the trailers
        are ready to be consumed.
        """
        result = componentize_py_runtime.call_import(54, [self], 1)
        return result[0]

    def get(self) -> Optional[Result[Fields, Error]]:
        """
        Retrieve reference to trailers, if they are ready.
        """
        result = componentize_py_runtime.call_import(55, [self], 1)
        return result[0]

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class OutgoingResponse:
    
    def __init__(self, status_code: int, headers: Fields):
        tmp = componentize_py_runtime.call_import(56, [status_code, headers], 1)[0]
        (_, func, args, _) = tmp.finalizer.detach()
        self.handle = tmp.handle
        self.finalizer = weakref.finalize(self, func, args[0], args[1])

    def write(self) -> OutgoingBody:
        """
        Will give the child outgoing-response at most once. subsequent calls will
        return an error.
        """
        result = componentize_py_runtime.call_import(57, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class OutgoingBody:
    
    def write(self) -> streams.OutputStream:
        """
        Will give the child output-stream at most once. subsequent calls will
        return an error.
        """
        result = componentize_py_runtime.call_import(58, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    @staticmethod
    def finish(this: OutgoingBody, trailers: Optional[Fields]) -> None:
        """
        Finalize an outgoing body, optionally providing trailers. This must be
        called to signal that the response is complete. If the `outgoing-body` is
        dropped without calling `outgoing-body-finalize`, the implementation
        should treat the body as corrupted.
        """
        result = componentize_py_runtime.call_import(59, [this, trailers], 0)
        return

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class FutureIncomingResponse:
    """
    The following block defines a special resource type used by the
    `wasi:http/outgoing-handler` interface to emulate
    `future<result<response, error>>` in advance of Preview3. Given a
    `future-incoming-response`, the client can call the non-blocking `get`
    method to get the result if it is available. If the result is not available,
    the client can call `listen` to get a `pollable` that can be passed to
    `wasi:io/poll.poll-list`.
    """
    
    def get(self) -> Optional[Result[Result[IncomingResponse, Error], None]]:
        """
        option indicates readiness.
        outer result indicates you are allowed to get the
        incoming-response-or-error at most once. subsequent calls after ready
        will return an error here.
        inner result indicates whether the incoming-response was available, or an
        error occured.
        """
        result = componentize_py_runtime.call_import(60, [self], 1)
        return result[0]

    def subscribe(self) -> poll.Pollable:
        result = componentize_py_runtime.call_import(61, [self], 1)
        return result[0]

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])



