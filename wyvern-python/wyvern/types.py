#!/usr/bin/env python3
#
# Copyright 2018 | Dario Ostuni <dario.ostuni@gmail.com>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from enum import Enum
from .util import UnreachableError


class DataType(Enum):
    int32 = "I32"
    uint32 = "U32"
    float32 = "F32"
    bool = "Bool"


class IoType(Enum):
    input = "input"
    output = "output"
    private = "private"


class Variable:
    pass


class Array:
    def _get_key(self, key):
        if type(key) == int:
            key = Constant.uint32(key, self._ctx)
        elif type(key) == Constant and key._ty == DataType.uint32:
            pass
        else:
            raise TypeError
        return key

    def __setitem__(self, key, value):
        key = self._get_key(key)
        value = self._ctx._sanitize(value)
        if type(value) != Constant or value._ty != self._ty:
            raise TypeError
        self._ctx.getProgramBuilder()._add_command({
            "ArrayStore": [self._tid, key._tid, value._tid]
        })

    def __getitem__(self, key):
        key = self._get_key(key)
        element = Constant._new_constant(self._ctx, self._ty)
        self._ctx.getProgramBuilder()._add_command({
            "ArrayLoad": [element._tid, self._tid, key._tid]
        })
        return element

    def __len__(self):
        length = Constant._new_constant(self._ctx, DataType.uint32)
        self._ctx.getProgramBuilder()._add_command({
            "ArrayLen": [length._tid, self._tid]
        })
        return length


class Constant:
    def getContext(self):
        return self._ctx

    def getProgramBuilder(self):
        return self.getContext().getProgramBuilder()

    def __add__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Add": [result._tid, self._tid, other._tid]
        })
        return result

    def __sub__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Sub": [result._tid, self._tid, other._tid]
        })
        return result

    def __mul__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Mul": [result._tid, self._tid, other._tid]
        })
        return result

    def __floordiv__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Div": [result._tid, self._tid, other._tid]
        })
        return result

    def __truediv__(self, other):
        return self.__floordiv__(other)

    def __mod__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Rem": [result._tid, self._tid, other._tid]
        })
        return result

    def __inv__(self):
        assert self._ty in (DataType.int32, DataType.uint32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Not": [result._tid, self._tid]
        })
        return result

    def not_(self):
        assert self._ty in (DataType.bool,)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Not": [result._tid, self._tid]
        })
        return result

    def __neg__(self):
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Neg": [result._tid, self._tid]
        })
        return result

    def __lshift__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Shl": [result._tid, self._tid, other._tid]
        })
        return result

    def __rshift__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "Shr": [result._tid, self._tid, other._tid]
        })
        return result

    def __xor__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.bool)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "BitXor": [result._tid, self._tid, other._tid]
        })
        return result

    def __and__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.bool)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "BitAnd": [result._tid, self._tid, other._tid]
        })
        return result

    def __or__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.bool)
        result = Constant._new_constant(self.getContext(), self._ty)
        self.getProgramBuilder()._add_command({
            "BitOr": [result._tid, self._tid, other._tid]
        })
        return result

    def __eq__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32,
                            DataType.float32, DataType.bool)
        result = Constant._new_constant(self.getContext(), DataType.bool)
        self.getProgramBuilder()._add_command({
            "Eq": [result._tid, self._tid, other._tid]
        })
        return result

    def __ne__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32,
                            DataType.float32, DataType.bool)
        result = Constant._new_constant(self.getContext(), DataType.bool)
        self.getProgramBuilder()._add_command({
            "Ne": [result._tid, self._tid, other._tid]
        })
        return result

    def __lt__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), DataType.bool)
        self.getProgramBuilder()._add_command({
            "Lt": [result._tid, self._tid, other._tid]
        })
        return result

    def __le__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), DataType.bool)
        self.getProgramBuilder()._add_command({
            "Le": [result._tid, self._tid, other._tid]
        })
        return result

    def __gt__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), DataType.bool)
        self.getProgramBuilder()._add_command({
            "Gt": [result._tid, self._tid, other._tid]
        })
        return result

    def __ge__(self, other):
        other = self._ctx._sanitize(other)
        assert self.getProgramBuilder() == other.getProgramBuilder()
        assert self._ty == other._ty
        assert self._ty in (DataType.int32, DataType.uint32, DataType.float32)
        result = Constant._new_constant(self.getContext(), DataType.bool)
        self.getProgramBuilder()._add_command({
            "Ge": [result._tid, self._tid, other._tid]
        })
        return result

    @staticmethod
    def _new_constant(ctx, ty):
        const = Constant()
        const._ctx = ctx
        const._ty = ty
        const._tid = ctx._new_constant(ty)
        return const

    @staticmethod
    def int32(value, ctx=None):
        if type(value) not in (int, Constant):
            value = int(value)
        if type(value) == Constant:
            ctx = value.getContext()
        const = Constant._new_constant(ctx, DataType.int32)
        p = ctx.getProgramBuilder()
        if type(value) == int:
            if value < 2**31 or value >= 2**31:
                raise ValueError
            p._add_command({
                "Constant": [const._tid, {DataType.int32.value: value}]
            })
        elif type(value) == Constant:
            assert p == value.getProgramBuilder()
            if value._ty == DataType.int32:
                const._tid = value._tid
            elif value._ty == DataType.uint32:
                p._add_command({"I32fromU32": [const._tid, value._tid]})
            elif value._ty == DataType.float32:
                p._add_command({"I32fromF32": [const._tid, value._tid]})
            else:
                raise TypeError
        else:
            raise UnreachableError
        return const

    @staticmethod
    def uint32(value, ctx=None):
        if type(value) not in (int, Constant):
            value = int(value)
        if type(value) == Constant:
            ctx = value.getContext()
        const = Constant._new_constant(ctx, DataType.uint32)
        p = ctx.getProgramBuilder()
        if type(value) == int:
            if value < 0 or value >= 2**32:
                raise ValueError
            p._add_command({
                "Constant": [const._tid, {DataType.uint32.value: value}]
            })
        elif type(value) == Constant:
            assert p == value.getProgramBuilder()
            if value._ty == DataType.uint32:
                const._tid = value._tid
            elif value._ty == DataType.int32:
                p._add_command({"U32fromI32": [const._tid, value._tid]})
            elif value._ty == DataType.float32:
                p._add_command({"U32fromF32": [const._tid, value._tid]})
            else:
                raise TypeError
        else:
            raise UnreachableError
        return const

    @staticmethod
    def float32(value, ctx=None):
        if type(value) not in (float, Constant):
            value = float(value)
        if type(value) == Constant:
            ctx = value.getContext()
        const = Constant._new_constant(ctx, DataType.float32)
        p = ctx.getProgramBuilder()
        if type(value) == float:
            p._add_command({
                "Constant": [const._tid, {DataType.float32.value: value}]
            })
        elif type(value) == Constant:
            assert p == value.getProgramBuilder()
            if value._ty == DataType.uint32:
                p._add_command({"F32fromU32": [const._tid, value._tid]})
            elif value._ty == DataType.int32:
                p._add_command({"F32fromI32": [const._tid, value._tid]})
            elif value._ty == DataType.float32:
                const._tid = value._tid
            else:
                raise TypeError
        else:
            raise UnreachableError
        return const

    @staticmethod
    def bool(value, ctx=None):
        if type(value) not in (bool, Constant):
            value = bool(value)
        if type(value) == Constant:
            ctx = value.getContext()
        const = Constant._new_constant(ctx, DataType.bool)
        p = ctx.getProgramBuilder()
        if type(value) == bool:
            p._add_command({
                "Constant": [const._tid, {DataType.bool.value: value}]
            })
        elif type(value) == Constant:
            assert p == value.getProgramBuilder()
            if value._ty == DataType.bool:
                const._tid = value._tid
            else:
                raise TypeError
        else:
            raise UnreachableError
        return const
