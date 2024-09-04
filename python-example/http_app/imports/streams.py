"""
WASI I/O is an I/O abstraction API which is currently focused on providing
stream types.

In the future, the component model is expected to add built-in stream types;
when it does, they are expected to subsume this API.
"""
from typing import TypeVar, Generic, Union, Optional, Union, Protocol, Tuple, List, Any
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from ..types import Result, Ok, Err, Some
import componentize_py_runtime
from ..imports import poll

class Error:
    """
    Contextual error information about the last failure that happened on
    a read, write, or flush from an `input-stream` or `output-stream`.
    
    This type is returned through the `stream-error` type whenever an
    operation on a stream directly fails or an error is discovered
    after-the-fact, for example when a write's failure shows up through a
    later `flush` or `check-write`.
    
    Interfaces such as `wasi:filesystem/types` provide functionality to
    further "downcast" this error into interface-specific error information.
    """
    
    def to_debug_string(self) -> str:
        """
        Returns a string that's suitable to assist humans in debugging this
        error.
        
        The returned string will change across platforms and hosts which
        means that parsing it, for example, would be a
        platform-compatibility hazard.
        """
        result = componentize_py_runtime.call_import(6, [self], 1)
        return result[0]

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])



@dataclass
class StreamErrorLastOperationFailed:
    value: Error


@dataclass
class StreamErrorClosed:
    pass


# An error for input-stream and output-stream operations.
StreamError = Union[StreamErrorLastOperationFailed, StreamErrorClosed]

class InputStream:
    """
    An input bytestream.
    
    `input-stream`s are *non-blocking* to the extent practical on underlying
    platforms. I/O operations always return promptly; if fewer bytes are
    promptly available than requested, they return the number of bytes promptly
    available, which could even be zero. To wait for data to be available,
    use the `subscribe` function to obtain a `pollable` which can be polled
    for using `wasi:io/poll`.
    """
    
    def read(self, len: int) -> bytes:
        """
        Perform a non-blocking read from the stream.
        
        This function returns a list of bytes containing the data that was
        read, along with a `stream-status` which, indicates whether further
        reads are expected to produce data. The returned list will contain up to
        `len` bytes; it may return fewer than requested, but not more. An
        empty list and `stream-status:open` indicates no more data is
        available at this time, and that the pollable given by `subscribe`
        will be ready when more data is available.
        
        Once a stream has reached the end, subsequent calls to `read` or
        `skip` will always report `stream-status:ended` rather than producing more
        data.
        
        When the caller gives a `len` of 0, it represents a request to read 0
        bytes. This read should  always succeed and return an empty list and
        the current `stream-status`.
        
        The `len` parameter is a `u64`, which could represent a list of u8 which
        is not possible to allocate in wasm32, or not desirable to allocate as
        as a return value by the callee. The callee may return a list of bytes
        less than `len` in size while more bytes are available for reading.
        """
        result = componentize_py_runtime.call_import(7, [self, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def blocking_read(self, len: int) -> bytes:
        """
        Read bytes from a stream, after blocking until at least one byte can
        be read. Except for blocking, identical to `read`.
        """
        result = componentize_py_runtime.call_import(8, [self, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def skip(self, len: int) -> int:
        """
        Skip bytes from a stream.
        
        This is similar to the `read` function, but avoids copying the
        bytes into the instance.
        
        Once a stream has reached the end, subsequent calls to read or
        `skip` will always report end-of-stream rather than producing more
        data.
        
        This function returns the number of bytes skipped, along with a
        `stream-status` indicating whether the end of the stream was
        reached. The returned value will be at most `len`; it may be less.
        """
        result = componentize_py_runtime.call_import(9, [self, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def blocking_skip(self, len: int) -> int:
        """
        Skip bytes from a stream, after blocking until at least one byte
        can be skipped. Except for blocking behavior, identical to `skip`.
        """
        result = componentize_py_runtime.call_import(10, [self, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def subscribe(self) -> poll.Pollable:
        """
        Create a `pollable` which will resolve once either the specified stream
        has bytes available to read or the other end of the stream has been
        closed.
        The created `pollable` is a child resource of the `input-stream`.
        Implementations may trap if the `input-stream` is dropped before
        all derived `pollable`s created with this function are dropped.
        """
        result = componentize_py_runtime.call_import(11, [self], 1)
        return result[0]

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])


class OutputStream:
    """
    An output bytestream.
    
    `output-stream`s are *non-blocking* to the extent practical on
    underlying platforms. Except where specified otherwise, I/O operations also
    always return promptly, after the number of bytes that can be written
    promptly, which could even be zero. To wait for the stream to be ready to
    accept data, the `subscribe` function to obtain a `pollable` which can be
    polled for using `wasi:io/poll`.
    """
    
    def check_write(self) -> int:
        """
        Check readiness for writing. This function never blocks.
        
        Returns the number of bytes permitted for the next call to `write`,
        or an error. Calling `write` with more bytes than this function has
        permitted will trap.
        
        When this function returns 0 bytes, the `subscribe` pollable will
        become ready when this function will report at least 1 byte, or an
        error.
        """
        result = componentize_py_runtime.call_import(12, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def write(self, contents: bytes) -> None:
        """
        Perform a write. This function never blocks.
        
        Precondition: check-write gave permit of Ok(n) and contents has a
        length of less than or equal to n. Otherwise, this function will trap.
        
        returns Err(closed) without writing if the stream has closed since
        the last call to check-write provided a permit.
        """
        result = componentize_py_runtime.call_import(13, [self, contents], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def blocking_write_and_flush(self, contents: bytes) -> None:
        """
        Perform a write of up to 4096 bytes, and then flush the stream. Block
        until all of these operations are complete, or an error occurs.
        
        This is a convenience wrapper around the use of `check-write`,
        `subscribe`, `write`, and `flush`, and is implemented with the
        following pseudo-code:
        
        ```text
        let pollable = this.subscribe();
        while !contents.is_empty() {
        // Wait for the stream to become writable
        poll-one(pollable);
        let Ok(n) = this.check-write(); // eliding error handling
        let len = min(n, contents.len());
        let (chunk, rest) = contents.split_at(len);
        this.write(chunk  );            // eliding error handling
        contents = rest;
        }
        this.flush();
        // Wait for completion of `flush`
        poll-one(pollable);
        // Check for any errors that arose during `flush`
        let _ = this.check-write();         // eliding error handling
        ```
        """
        result = componentize_py_runtime.call_import(14, [self, contents], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def flush(self) -> None:
        """
        Request to flush buffered output. This function never blocks.
        
        This tells the output-stream that the caller intends any buffered
        output to be flushed. the output which is expected to be flushed
        is all that has been passed to `write` prior to this call.
        
        Upon calling this function, the `output-stream` will not accept any
        writes (`check-write` will return `ok(0)`) until the flush has
        completed. The `subscribe` pollable will become ready when the
        flush has completed and the stream can accept more writes.
        """
        result = componentize_py_runtime.call_import(15, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def blocking_flush(self) -> None:
        """
        Request to flush buffered output, and block until flush completes
        and stream is ready for writing again.
        """
        result = componentize_py_runtime.call_import(16, [self], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def subscribe(self) -> poll.Pollable:
        """
        Create a `pollable` which will resolve once the output-stream
        is ready for more writing, or an error has occured. When this
        pollable is ready, `check-write` will return `ok(n)` with n>0, or an
        error.
        
        If the stream is closed, this pollable is always ready immediately.
        
        The created `pollable` is a child resource of the `output-stream`.
        Implementations may trap if the `output-stream` is dropped before
        all derived `pollable`s created with this function are dropped.
        """
        result = componentize_py_runtime.call_import(17, [self], 1)
        return result[0]

    def write_zeroes(self, len: int) -> None:
        """
        Write zeroes to a stream.
        
        this should be used precisely like `write` with the exact same
        preconditions (must use check-write first), but instead of
        passing a list of bytes, you simply pass the number of zero-bytes
        that should be written.
        """
        result = componentize_py_runtime.call_import(18, [self, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def blocking_write_zeroes_and_flush(self, len: int) -> None:
        """
        Perform a write of up to 4096 zeroes, and then flush the stream.
        Block until all of these operations are complete, or an error
        occurs.
        
        This is a convenience wrapper around the use of `check-write`,
        `subscribe`, `write-zeroes`, and `flush`, and is implemented with
        the following pseudo-code:
        
        ```text
        let pollable = this.subscribe();
        while num_zeroes != 0 {
        // Wait for the stream to become writable
        poll-one(pollable);
        let Ok(n) = this.check-write(); // eliding error handling
        let len = min(n, num_zeroes);
        this.write-zeroes(len);         // eliding error handling
        num_zeroes -= len;
        }
        this.flush();
        // Wait for completion of `flush`
        poll-one(pollable);
        // Check for any errors that arose during `flush`
        let _ = this.check-write();         // eliding error handling
        ```
        """
        result = componentize_py_runtime.call_import(19, [self, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def splice(self, src: InputStream, len: int) -> int:
        """
        Read from one stream and write to another.
        
        This function returns the number of bytes transferred; it may be less
        than `len`.
        
        Unlike other I/O functions, this function blocks until all the data
        read from the input stream has been written to the output stream.
        """
        result = componentize_py_runtime.call_import(20, [self, src, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def blocking_splice(self, src: InputStream, len: int) -> int:
        """
        Read from one stream and write to another, with blocking.
        
        This is similar to `splice`, except that it blocks until at least
        one byte can be read.
        """
        result = componentize_py_runtime.call_import(21, [self, src, len], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def forward(self, src: InputStream) -> int:
        """
        Forward the entire contents of an input stream to an output stream.
        
        This function repeatedly reads from the input stream and writes
        the data to the output stream, until the end of the input stream
        is reached, or an error is encountered.
        
        Unlike other I/O functions, this function blocks until the end
        of the input stream is seen and all the data has been written to
        the output stream.
        
        This function returns the number of bytes transferred, and the status of
        the output stream.
        """
        result = componentize_py_runtime.call_import(22, [self, src], 1)
        if isinstance(result[0], Err):
            raise result[0]
        else:
            return result[0].value

    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])



