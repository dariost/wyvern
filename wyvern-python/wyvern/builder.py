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

import json
from .types import Variable, Array, Constant, IoType, DataType
from .util import UnreachableError
from copy import deepcopy


class ProgramBuilder:
    def __init__(self):
        self._program = {}
        self._program["symbol"] = {}
        self._program["operation"] = []
        self._program["storage"] = {}
        self._program["input"] = {}
        self._program["output"] = {}
        self._label_id = 1
        self._token_id = 0
        self._stack = [[]]

    def newContext(self):
        ctx = Context()
        ctx.__dict__["_builder"] = self
        ctx.__dict__["_var"] = {}
        return ctx

    def finalize(self):
        assert len(self._stack) == 1
        program = deepcopy(self._program)
        program["operation"].extend(deepcopy(self._stack[0]))
        return json.dumps(program)

    def _next_token_id(self):
        tid = self._token_id
        self._token_id += 1
        return tid

    def _next_label_id(self):
        lid = self._label_id
        self._label_id += 1
        return lid

    def _new_constant(self, ty):
        tid = self._next_token_id()
        self._program["symbol"][str(tid)] = {"Constant": ty.value}
        return tid

    def _new_variable(self, ty):
        tid = self._next_token_id()
        self._program["symbol"][str(tid)] = {"Variable": ty.value}
        return tid

    def _new_array(self, ty):
        tid = self._next_token_id()
        self._program["symbol"][str(tid)] = {"Array": ty.value}
        return tid

    def _add_command(self, cmd):
        self._stack[-1].append(cmd)

    def _push_stack(self):
        self._stack.append([])

    def _pop_stack(self):
        return self._stack.pop()


class Context:
    def If(self, cond, body):
        self._builder._push_stack()
        condition = cond()
        if type(condition) != Constant or condition._ty != DataType.bool:
            raise TypeError
        cond = self._builder._pop_stack()
        self._builder._push_stack()
        body()
        body = self._builder._pop_stack()
        self._builder._add_command({
            "If": [cond, condition._tid, self._builder._next_label_id(),
                   body, self._builder._next_label_id()]
        })

    def IfElse(self, cond, body1, body2):
        self._builder._push_stack()
        condition = cond()
        if type(condition) != Constant or condition._ty != DataType.bool:
            raise TypeError
        cond = self._builder._pop_stack()
        self._builder._push_stack()
        body1()
        body1 = self._builder._pop_stack()
        self._builder._push_stack()
        body2()
        body2 = self._builder._pop_stack()
        self._builder._add_command({
            "IfElse": [cond, condition._tid, self._builder._next_label_id(),
                       body1, self._builder._next_label_id(),
                       body2, self._builder._next_label_id()]
        })

    def While(self, cond, body):
        self._builder._push_stack()
        condition = cond()
        if type(condition) != Constant or condition._ty != DataType.bool:
            raise TypeError
        cond = self._builder._pop_stack()
        self._builder._push_stack()
        body()
        body = self._builder._pop_stack()
        self._builder._add_command({
            "While": [self._builder._next_label_id(), cond, condition._tid,
                      self._builder._next_label_id(), body,
                      self._builder._next_label_id()]

        })

    def workerId(self):
        wid = Constant._new_constant(self, DataType.uint32)
        self._builder._add_command({
            "WorkerId": wid._tid
        })
        return wid

    def numWorkers(self):
        wnum = Constant._new_constant(self, DataType.uint32)
        self._builder._add_command({
            "NumWorkers": wnum._tid
        })
        return wnum

    def getProgramBuilder(self):
        return self._builder

    def declVariable(self, name, ty, io_type):
        if name in self._var:
            raise NameError
        tid = self._new_variable(ty, name)
        if io_type == IoType.private:
            pass
        elif io_type in (IoType.input, IoType.output):
            self._builder._program[io_type.value][name] = tid
        else:
            raise UnreachableError

    def declArray(self, name, ty, io_type, size, max_size=0):
        size = self._sanitize(size)
        if name in self._var:
            raise NameError
        if type(size) != Constant or size._ty != DataType.uint32:
            raise TypeError
        tid = self._new_array(ty, name)
        self._builder._add_command({
            "ArrayNew": [tid, size._tid, ty.value,
                         max_size, io_type != IoType.private]
        })
        if io_type in (IoType.input, IoType.output):
            array_type = "SharedArray"
            self._builder._program[io_type.value][name] = tid
        else:
            array_type = "PrivateArray"
        self._builder._program["storage"][str(tid)] = {
            array_type: [ty.value, max_size]
        }

    def _get_array_object(self, name):
        array = Array()
        array._ctx = self
        array._ty = self._var[name][1]
        array._name = name
        array._tid = self._var[name][0]
        return array

    def _new_constant(self, ty):
        return self._builder._new_constant(ty)

    def _new_variable(self, ty, name):
        tid = self._builder._new_variable(ty)
        self._var[name] = (tid, ty, Variable)
        self._builder._program["storage"][str(tid)] = {
            "Variable": ty.value
        }
        return tid

    def _new_array(self, ty, name):
        tid = self._builder._new_array(ty)
        self._var[name] = (tid, ty, Array)
        return tid

    def _get(self, name):
        if name not in self._var:
            return None
        tid, ty, pyty = self._var[name]
        if pyty == Variable:
            const = Constant._new_constant(self, ty)
            self._builder._add_command({"Load": [const._tid, tid]})
            return const
        elif pyty == Array:
            return self._get_array_object(name)
        raise UnreachableError

    def _sanitize(self, value):
        if type(value) == Constant:
            pass
        elif type(value) == int:
            if value >= 0:
                value = Constant.uint32(value, self)
            else:
                value = Constant.int32(value, self)
        elif type(value) == float:
            value = Constant.float32(value, self)
        elif type(value) == bool:
            value = Constant.bool(value, self)
        else:
            raise TypeError
        return value

    def _set(self, name, value):
        value = self._sanitize(value)
        if name in self._var:
            assert value._ty == self._var[name][1]
            self._builder._add_command({
                "Store": [self._var[name][0], value._tid]
            })
        else:
            var = self._new_variable(value._ty, name)
            self._builder._add_command({"Store": [var, value._tid]})

    def __setattr__(self, name, value):
        self._set(name, value)

    def __getattr__(self, name):
        value = self._get(name)
        if value is None:
            raise AttributeError
        return value

    def __setitem__(self, key, value):
        self._set(key, value)

    def __getitem__(self, key):
        value = self._get(key)
        if value is None:
            raise KeyError
        return value
