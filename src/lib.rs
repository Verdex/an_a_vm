
pub mod error;
pub mod data;

use crate::error::*;
use crate::data::*;

use std::borrow::Cow;


pub struct Vm<T, S> {
    funs : Vec<Fun>,
    ops : Vec<GenOp<T, S>>,
    globals: Vec<S>,
}

struct Frame {
    fun_id : usize,
    ip : usize,
}

enum Coroutine<T> {
    Active {
        locals : Vec<T>,
        ip : usize,
        fun : usize,
        coroutines : Vec<Coroutine<T>>,
    },
    Finished,
}

impl<T : Clone, S> Vm<T, S> {
    pub fn new(funs : Vec<Fun>, ops : Vec<GenOp<T, S>>) -> Self {
        Vm { funs, ops, globals: vec![] }
    }

    pub fn with_globals(&mut self, globals: Vec<S>) -> Vec<S> { 
        std::mem::replace(&mut self.globals, globals)
    }

    pub fn run(&mut self, entry : usize) -> Result<Option<T>, VmError> {
        let mut frames : Vec<Frame> = vec![];
        let mut current = Frame { fun_id: entry, ip: 0 };

        let mut locals : Vec<Vec<T>> = vec![]; 
        let mut ret : Option<T> = None;
        let mut branch : bool = false;
        let mut dyn_call : Option<usize> = None;

        let mut coroutines : Vec<Vec<Coroutine<T>>> = vec![];

        // Note:  Initial locals for entry function
        locals.push(vec![]);
        coroutines.push(vec![]);
        loop {
            if current.fun_id >= self.funs.len() {
                return Err(VmError::FunDoesNotExist(fun, stack_trace(&current, &frames, &self.funs)));
            }

            if current.ip >= self.funs[current.fun_id].instrs.len() {
                // Note:  if the current function isn't pushed onto the return stack, then the
                // stack trace will leave out the current function where the problem is occurring.
                fun_stack.push(RetAddr { fun, instr: ip });
                return Err(VmError::InstrPointerOutOfRange(ip, stack_trace(&current, &frames, &self.funs)));
            }

            match self.funs[current.fun_id].instrs[current.ip] {
                Op::Gen(op_index, ref params) if op_index < self.ops.len() => {
                    let env = OpEnv { 
                        locals: &mut locals, 
                        globals: &mut self.globals,
                        ret: &mut ret, 
                        branch: &mut branch, 
                        dyn_call: &mut dyn_call,
                    };
                    match (self.ops[op_index].op)(env, params) {
                        Ok(()) => { },
                        Err(e) => { 
                            let name = self.ops[op_index].name.clone();
                            fun_stack.push(RetAddr { fun, instr: ip });
                            return Err(VmError::GenOpError(name, e, stack_trace(&current, &frames, &self.funs))); 
                        }
                    }
                    current.ip += 1;
                },
                Op::Gen(op_index, _) => {
                    // Note:  Indicate current function for stack trace.
                    fun_stack.push(RetAddr { fun, instr: ip });
                    return Err(VmError::GenOpDoesNotExist(op_index, stack_trace(&current, &frames, &self.funs)));
                },
                Op::Branch(target) if branch => {
                    current.ip = target;
                },
                Op::Branch(_) => { 
                    current.ip += 1;
                },
                Op::Call(fun_index, ref params) => {
                    fun_stack.push(RetAddr { fun, instr: ip + 1 });
                    fun = fun_index;
                    ip = 0;
                    let mut new_locals = vec![];
                    for param in params {
                        match get_local(*param, Cow::Borrowed(locals.last().unwrap())) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                fun_stack.push(RetAddr{ fun, instr: ip });
                                return Err(f(stack_trace(&current, &frames, &self.funs)));
                            },
                        }
                    }
                    locals.push(new_locals);
                    coroutines.push(vec![]);
                },
                Op::DynCall(ref params) if dyn_call.is_some() => {
                    fun_stack.push(RetAddr { fun, instr: ip + 1 });
                    fun = dyn_call.unwrap(); 
                    ip = 0;
                    let mut new_locals = vec![];
                    for param in params {
                        match get_local(*param, Cow::Borrowed(locals.last().unwrap())) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                fun_stack.push(RetAddr{ fun, instr: ip });
                                return Err(f(stack_trace(&current, &frames, &self.funs)));
                            },
                        }
                    }
                    locals.push(new_locals);
                    coroutines.push(vec![]);
                },
                Op::DynCall(_) => {
                    fun_stack.push(RetAddr { fun, instr: ip });
                    return Err(VmError::DynFunDoesNotExist(stack_trace(&current, &frames, &self.funs)));
                },
                Op::ReturnLocal(slot) => {
                    coroutines.pop().unwrap();
                    let current_locals = locals.pop().unwrap();

                    let ret_target = match get_local(slot, Cow::Owned(current_locals)) {
                        Ok(v) => v,
                        Err(f) => { 
                            fun_stack.push(RetAddr{ fun, instr: ip });
                            return Err(f(stack_trace(&current, &frames, &self.funs)));
                        },
                    };

                    match fun_stack.pop() {
                        // Note:  if the stack is empty then all execution is finished
                        None => {
                            return Ok(Some(ret_target));
                        },
                        Some(ret_addr) => {
                            fun = ret_addr.fun;
                            ip = ret_addr.instr;
                            ret = Some(ret_target);
                        },
                    }
                },
                Op::Return => {
                    match fun_stack.pop() {
                        // Note:  if the stack is empty then all execution is finished
                        None => {
                            return Ok(None);
                        },
                        Some(ret_addr) => {
                            coroutines.pop().unwrap();
                            locals.pop().unwrap();
                            fun = ret_addr.fun;
                            ip = ret_addr.instr;
                            ret = None;
                        },
                    }
                },
                Op::Yield(slot) => {
                    let current_coroutines = coroutines.pop().unwrap();
                    let current_locals = locals.pop().unwrap();
                    let current_ip = ip + 1;
                    let current_fun = fun;

                    let ret_target = match get_local(slot, Cow::Borrowed(&current_locals)) {
                        Ok(v) => v,
                        Err(f) => { 
                            fun_stack.push(RetAddr{ fun, instr: ip });
                            return Err(f(stack_trace(&current, &frames, &self.funs)));
                        },
                    };

                    let this_coroutine = Coroutine::Active {
                        coroutines: current_coroutines,
                        locals: current_locals,
                        ip: current_ip,
                        fun: current_fun,
                    };

                    coroutines.last_mut().unwrap().push(this_coroutine);

                    match fun_stack.pop() {
                        None => {
                            // Note: Top level yields are not supported.
                            return Err(VmError::TopLevelYield(ip)); 
                        },
                        Some(ret_addr) => {
                            fun = ret_addr.fun;
                            ip = ret_addr.instr;
                            ret = Some(ret_target);
                        },
                    }
                },
                Op::Finish => {
                    match fun_stack.pop() {
                        None => {
                            // Note: Top level yields are not supported.
                            return Err(VmError::TopLevelYield(ip)); 
                        },
                        Some(ret_addr) => {
                            coroutines.pop().unwrap();
                            locals.pop().unwrap();

                            fun = ret_addr.fun;
                            ip = ret_addr.instr;
                            ret = None;

                            coroutines.last_mut().unwrap().push(Coroutine::Finished);
                        },
                    }
                },
                Op::Resume(coroutine) if coroutine < coroutines.last().unwrap().len() => {
                    match coroutines.last_mut().unwrap().remove(coroutine) { 
                        Coroutine::Active { locals: c_locals, ip: c_ip, fun: c_fun, coroutines: c_cs } => {
                            fun_stack.push(RetAddr { fun, instr: ip + 1 });
                            fun = c_fun;
                            ip = c_ip;
                            locals.push(c_locals);
                            coroutines.push(c_cs);
                        },
                        Coroutine::Finished => {
                            fun_stack.push(RetAddr{ fun, instr: ip });
                            return Err(VmError::ResumeFinishedCoroutine(coroutine, stack_trace(&current, &frames, &self.funs)))
                        },
                    }
                },
                Op::Resume(coroutine) => {
                    fun_stack.push(RetAddr{ fun, instr: ip });
                    return Err(VmError::AccessMissingCoroutine(coroutine, stack_trace(&current, &frames, &self.funs)));
                },
                Op::FinishSetBranch(coroutine) if coroutine < coroutines.last().unwrap().len() => {
                    match coroutines.last().unwrap()[coroutine] {
                        Coroutine::Finished => { 
                            branch = true; 
                            coroutines.last_mut().unwrap().remove(coroutine);
                        },
                        Coroutine::Active { .. } => { branch = false; },
                    }
                    current.ip += 1;
                },
                Op::FinishSetBranch(coroutine) => { 
                    fun_stack.push(RetAddr{ fun, instr: ip });
                    return Err(VmError::AccessMissingCoroutine(coroutine, stack_trace(&current, &frames, &self.funs)));
                },
                Op::Drop(local) if local < locals.last().unwrap().len() => {
                    locals.last_mut().unwrap().remove(local);
                    current.ip += 1;
                },
                Op::Drop(local) => {
                    fun_stack.push(RetAddr{ fun, instr: ip });
                    return Err(VmError::AccessMissingLocal(local, stack_trace(&current, &frames, &self.funs)));
                },
                Op::Dup(local) if local < locals.last().unwrap().len() => {
                    let target = locals.last_mut().unwrap()[local].clone();
                    locals.last_mut().unwrap().push(target);
                    current.ip += 1;
                },
                Op::Dup(local) => {
                    fun_stack.push(RetAddr{ fun, instr: ip });
                    return Err(VmError::AccessMissingLocal(local, stack_trace(&current, &frames, &self.funs)));
                },
                Op::Swap(a, b) if a < locals.last().unwrap().len() && b < locals.last().unwrap().len() => {
                    locals.last_mut().unwrap().swap(a, b);
                    current.ip += 1;
                },
                Op::Swap(a, b) if b < locals.last().unwrap().len() => {
                    fun_stack.push(RetAddr{ fun, instr: ip });
                    return Err(VmError::AccessMissingLocal(a, stack_trace(&current, &frames, &self.funs)));
                },
                Op::Swap(_, b) => {
                    fun_stack.push(RetAddr{ fun, instr: ip });
                    return Err(VmError::AccessMissingLocal(b, stack_trace(&current, &frames, &self.funs)));
                },
                Op::PushRet if ret.is_some() => {
                    locals.last_mut().unwrap().push(ret.unwrap());
                    ret = None;
                    current.ip += 1;
                },
                Op::PushRet => {
                    fun_stack.push(RetAddr{ fun, instr: ip });
                    return Err(VmError::AccessMissingReturn(stack_trace(&current, &frames, &self.funs)));
                },
            }
        }
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

fn stack_trace(current : &Frame, stack : &[Frame], fun_map : &[Fun]) -> StackTrace {

    struct RetAddr { fun : usize, instr : usize }

    let mut stack = stack.iter().map(|x| RetAddr { fun: x.fun_id, instr: x.ip }).collect::<Vec<_>>();
    stack.push(RetAddr { fun: current.fun_id, instr: current.ip + 1});

    let mut trace = vec![];
    for addr in stack {
        // Note:  if the function was already pushed into the stack, then
        // that means that it already resolved to a known function.  Don't
        // have to check again that the fun map has it.
        let name = fun_map[addr.fun].name.clone();
        trace.push((name, addr.instr - 1));
    }
    trace
}
