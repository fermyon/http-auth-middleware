"""
A poll API intended to let users wait for I/O events on multiple handles
at once.
"""
from typing import TypeVar, Generic, Union, Optional, Union, Protocol, Tuple, List, Any
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from ..types import Result, Ok, Err, Some
import componentize_py_runtime


class Pollable:
    """
    A "pollable" handle.
    """
    
    def drop(self):
        (_, func, args, _) = self.finalizer.detach()
        self.handle = None
        func(args[0], args[1])



def poll_list(in_: List[Pollable]) -> List[int]:
    """
    Poll for completion on a set of pollables.
    
    This function takes a list of pollables, which identify I/O sources of
    interest, and waits until one or more of the events is ready for I/O.
    
    The result `list<u32>` contains one or more indices of handles in the
    argument list that is ready for I/O.
    
    If the list contains more elements than can be indexed with a `u32`
    value, this function traps.
    
    A timeout can be implemented by adding a pollable from the
    wasi-clocks API to the list.
    
    This function does not return a `result`; polling in itself does not
    do any I/O so it doesn't fail. If any of the I/O sources identified by
    the pollables has an error, it is indicated by marking the source as
    being reaedy for I/O.
    """
    result = componentize_py_runtime.call_import(1, [in_], 1)
    return result[0]

def poll_one(in_: Pollable) -> None:
    """
    Poll for completion on a single pollable.
    
    This function is similar to `poll-list`, but operates on only a single
    pollable. When it returns, the handle is ready for I/O.
    """
    result = componentize_py_runtime.call_import(2, [in_], 0)
    return

