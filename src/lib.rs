
pub mod error;
pub mod data;

use crate::error::*;
use crate::data::*;

use std::borrow::Cow;
use std::rc::Rc;

pub struct Vm<T, S> {
    funs : Vec<Fun<T>>,
    ops : Vec<GenOp<T, S>>,
    globals: Vec<S>,
    frames : Vec<Frame<T>>,
    current : Frame<T>,
}

impl<T : Clone, S> Vm<T, S> {
    pub fn new(funs : Vec<Fun<T>>, ops : Vec<GenOp<T, S>>) -> Self {
        let current = Frame { fun_id: 0, ip: 0, ret: None, branch: false, dyn_call: None, locals: vec![], coroutines: vec![] };
        Vm { funs, ops, globals: vec![], frames: vec![], current }
    }

    pub fn with_globals(&mut self, globals: Vec<S>) -> Vec<S> { 
        std::mem::replace(&mut self.globals, globals)
    }

    pub fn run(&mut self, entry : usize) -> Result<Option<T>, VmError> {
        self.current.fun_id = entry;

        loop {
            if self.current.fun_id >= self.funs.len() {
                return Err(VmError::FunDoesNotExist(self.current.fun_id, self.stack_trace()));
            }

            if self.current.ip >= self.funs[self.current.fun_id].instrs.len() {
                // Note:  if the current function isn't pushed onto the return stack, then the
                // stack trace will leave out the current function where the problem is occurring.
                return Err(VmError::InstrPointerOutOfRange(self.current.ip, self.stack_trace()));
            }

            match self.funs[self.current.fun_id].instrs[self.current.ip] {
                Op::Gen(op_index, ref params) if op_index < self.ops.len() => {
                    match &self.ops[op_index] {
                        GenOp::Vm { name, op } => {
                            let env = VmEnv { 
                                globals: &mut self.globals,
                                frames: &mut self.frames, 
                                current: &mut self.current,
                            };

                            match op(env, params) {
                                Ok(v) => { 
                                    self.current.ret = v;
                                },
                                Err(e) => {
                                    return Err(VmError::GenOpError(Rc::clone(name), e, self.stack_trace()));
                                },
                            }
                        },
                        GenOp::Global { name, op } => {
                            match op(&mut self.globals, params) {
                                Ok(v) => { 
                                    self.current.ret = v;
                                },
                                Err(e) => {
                                    return Err(VmError::GenOpError(Rc::clone(name), e, self.stack_trace()));
                                },
                            }
                        },
                        GenOp::Local { name, op } => {
                            match op(&mut self.current.locals, params) {
                                Ok(v) => { 
                                    self.current.ret = v;
                                },
                                Err(e) => {
                                    return Err(VmError::GenOpError(Rc::clone(name), e, self.stack_trace()));
                                },
                            }
                        },
                        GenOp::Frame { name, op } => {
                            match op(&mut self.current, params) {
                                Ok(v) => { 
                                    self.current.ret = v;
                                },
                                Err(e) => {
                                    return Err(VmError::GenOpError(Rc::clone(name), e, self.stack_trace()));
                                },
                            }
                        },
                    }
                    self.current.ip += 1;
                },
                Op::Gen(op_index, _) => {
                    // Note:  Indicate current function for stack trace.
                    return Err(VmError::GenOpDoesNotExist(op_index, self.stack_trace()));
                },
                Op::Branch(target) if self.current.branch => {
                    self.current.ip = target;
                },
                Op::Branch(_) => { 
                    self.current.ip += 1;
                },
                Op::Call(fun_index, ref params) => {
                    let mut new_locals = vec![];
                    for param in params {
                        match get_local(*param, Cow::Borrowed(&self.current.locals)) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                return Err(f(self.stack_trace()));
                            },
                        }
                    }
                    self.current.ip += 1;
                    let current = std::mem::replace(&mut self.current, Frame { fun_id: fun_index, ip: 0, ret: None, branch: false, dyn_call: None, locals: new_locals, coroutines: vec![] });
                    self.frames.push(current);
                },
                Op::DynCall(ref params) if self.current.dyn_call.is_some() => {
                    let mut new_locals = vec![];
                    for param in params {
                        match get_local(*param, Cow::Borrowed(&self.current.locals)) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                return Err(f(self.stack_trace()));
                            },
                        }
                    }
                    let target_fun_id = self.current.dyn_call.unwrap();
                    self.current.ip += 1;
                    let current = std::mem::replace(&mut self.current, Frame { fun_id: target_fun_id, ip: 0, ret: None, branch: false, dyn_call: None, locals: new_locals, coroutines: vec![] });
                    self.frames.push(current);
                },
                Op::DynCall(_) => {
                    return Err(VmError::DynFunDoesNotExist(self.stack_trace()));
                },
                Op::ReturnLocal(slot) => {
                    let current_locals = std::mem::replace(&mut self.current.locals, vec![]);

                    let ret_target = match get_local(slot, Cow::Owned(current_locals)) {
                        Ok(v) => v,
                        Err(f) => { 
                            return Err(f(self.stack_trace()));
                        },
                    };

                    match self.frames.pop() {
                        // Note:  if the stack is empty then all execution is finished
                        None => {
                            return Ok(Some(ret_target));
                        },
                        Some(frame) => {
                            self.current = frame;
                            self.current.ret = Some(ret_target);
                        },
                    }
                },
                Op::Return => {
                    match self.frames.pop() {
                        // Note:  if the stack is empty then all execution is finished
                        None => {
                            return Ok(None);
                        },
                        Some(frame) => {
                            self.current = frame;
                            self.current.ret = None;
                        },
                    }
                },
                Op::CoYield(slot) => {

                    let ret_target = match get_local(slot, Cow::Borrowed(&self.current.locals)) {
                        Ok(v) => v,
                        Err(f) => { 
                            return Err(f(self.stack_trace()));
                        },
                    };

                    match self.frames.pop() {
                        None => {
                            // Note: Top level yields are not supported.
                            return Err(VmError::TopLevelYield(self.current.ip)); 
                        },
                        Some(frame) => {
                            self.current.ip += 1;
                            let coroutine = std::mem::replace(&mut self.current, frame);
                            self.current.ret = Some(ret_target);
                            match self.current.coroutines.iter().position(co_is_running) {
                                Some(index) => {
                                    let _ = std::mem::replace(&mut self.current.coroutines[index], Coroutine::Active(coroutine));
                                },
                                None => { 
                                    self.current.coroutines.push(Coroutine::Active(coroutine));
                                },
                            }
                        },
                    }
                },
                Op::CoFinish => {
                    match self.frames.pop() {
                        None => {
                            // Note: Top level yields are not supported.
                            return Err(VmError::TopLevelYield(self.current.ip)); 
                        },
                        Some(frame) => {
                            self.current = frame;
                            self.current.ret = None;

                            match self.current.coroutines.iter().position(co_is_running) {
                                Some(index) => {
                                    let _ = std::mem::replace(&mut self.current.coroutines[index], Coroutine::Finished);
                                },
                                None => { 
                                    // TODO this represents a coroutine that has no yields and should have 
                                    // a test to go along with it
                                    self.current.coroutines.push(Coroutine::Finished);
                                },
                            }
                        },
                    }
                },
                Op::CoResume(coroutine) if coroutine < self.current.coroutines.len() => {
                    match std::mem::replace(&mut self.current.coroutines[coroutine], Coroutine::Running) { 
                        Coroutine::Active(frame) => {
                            self.current.ip += 1;
                            let old_current = std::mem::replace(&mut self.current, frame);
                            self.frames.push(old_current);
                        },
                        Coroutine::Finished => {
                            return Err(VmError::ResumeFinishedCoroutine(coroutine, self.stack_trace()))
                        },
                        Coroutine::Running => { unreachable!(); },
                    }
                },
                Op::CoResume(coroutine) => {
                    return Err(VmError::AccessMissingCoroutine(coroutine, self.stack_trace()));
                },
                Op::CoDrop(coroutine) if coroutine < self.current.coroutines.len() => {
                    self.current.coroutines.remove(coroutine);
                    self.current.ip += 1;
                },
                Op::CoDrop(coroutine) => {
                    return Err(VmError::AccessMissingCoroutine(coroutine, self.stack_trace()));
                },
                Op::CoDup(coroutine) if coroutine < self.current.coroutines.len() => {
                    let target = self.current.coroutines[coroutine].clone();
                    self.current.coroutines.push(target);
                    self.current.ip += 1;
                },
                Op::CoDup(coroutine) => {
                    return Err(VmError::AccessMissingCoroutine(coroutine, self.stack_trace()));
                },
                Op::CoSwap(a, b) if a < self.current.coroutines.len() && b < self.current.coroutines.len() => {
                    self.current.coroutines.swap(a, b);
                    self.current.ip += 1;
                },
                Op::CoSwap(a, b) if b < self.current.coroutines.len() => {
                    return Err(VmError::AccessMissingCoroutine(a, self.stack_trace()));
                },
                Op::CoSwap(_, b) => {
                    return Err(VmError::AccessMissingCoroutine(b, self.stack_trace()));
                },
                Op::Drop(local) if local < self.current.locals.len() => {
                    self.current.locals.remove(local);
                    self.current.ip += 1;
                },
                Op::Drop(local) => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Dup(local) if local < self.current.locals.len() => {
                    let target = self.current.locals[local].clone();
                    self.current.locals.push(target);
                    self.current.ip += 1;
                },
                Op::Dup(local) => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Swap(a, b) if a < self.current.locals.len() && b < self.current.locals.len() => {
                    self.current.locals.swap(a, b);
                    self.current.ip += 1;
                },
                Op::Swap(a, b) if b < self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(a, self.stack_trace()));
                },
                Op::Swap(_, b) => {
                    return Err(VmError::AccessMissingLocal(b, self.stack_trace()));
                },
                Op::PushRet if self.current.ret.is_some() => {
                    let ret = std::mem::replace(&mut self.current.ret, None);
                    self.current.locals.push(ret.unwrap());
                    self.current.ret = None;
                    self.current.ip += 1;
                },
                Op::PushRet => {
                    return Err(VmError::AccessMissingReturn(self.stack_trace()));
                },
                Op::PushLocal(ref t) => {
                    self.current.locals.push(t.clone());
                    self.current.ip += 1;
                }
            }
        }
    }

    fn stack_trace(&self) -> StackTrace {
        struct RetAddr { fun : usize, instr : usize }

        let mut stack = self.frames.iter().map(|x| RetAddr { fun: x.fun_id, instr: x.ip }).collect::<Vec<_>>();
        stack.push(RetAddr { fun: self.current.fun_id, instr: self.current.ip + 1});

        let mut trace = vec![];
        for addr in stack {
            // Note:  if the function was already pushed into the stack, then
            // that means that it already resolved to a known function.  Don't
            // have to check again that the fun map has it.
            let name = self.funs[addr.fun].name.clone();
            trace.push((name, addr.instr - 1));
        }
        trace
    }
}

fn get_local<T : Clone>(index: usize, locals : Cow<Vec<T>>) -> Result<T, Box<dyn Fn(StackTrace) -> VmError>> {
    if index >= locals.len() {
        Err(Box::new(move |trace| VmError::AccessMissingLocal(index, trace)))
    }
    else {
        match locals {
            Cow::Borrowed(locals) => Ok(locals[index].clone()),
            Cow::Owned(mut locals) => Ok(locals.swap_remove(index)),
        }
    }
}

fn co_is_running<T>(coroutine : &Coroutine<T>) -> bool {
    match coroutine { 
        Coroutine::Running => true,
        _ => false,
    }
}