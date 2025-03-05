
use std::borrow::Cow;

pub type StackTrace = Vec<(Box<str>, usize)>;

#[derive(Debug)]
pub enum VmError {
    FunDoesNotExist(usize, StackTrace),
    DynFunDoesNotExist(StackTrace),
    InstrPointerOutOfRange(usize, StackTrace),
    GenOpDoesNotExist(usize, StackTrace),
    AccessMissingReturn(StackTrace),
    AccessMissingLocal(usize, StackTrace),
    GenOpError(Box<str>, Box<dyn std::error::Error>, StackTrace),
    TopLevelYield(usize),
    AccessMissingCoroutine(usize, StackTrace),
    ResumeFinishedCoroutine(usize, StackTrace),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        fn d(x : &StackTrace) -> String {
            x.into_iter().map(|(n, i)| format!("    {} at index {}\n", n, i)).collect()
        }

        match self { 
            VmError::FunDoesNotExist(fun_index, trace) => 
                write!(f, "Fun Index {} does not exist: \n{}", fun_index, d(trace)),
            VmError::DynFunDoesNotExist(trace) => 
                write!(f, "Dynamic fun does not exist: \n{}", d(trace)),
            VmError::InstrPointerOutOfRange(instr, trace) => 
                write!(f, "Instr Index {} does not exist: \n{}", instr, d(trace)),
            VmError::GenOpDoesNotExist(op_index, trace) => 
                write!(f, "GenOp {} does not exist: \n{}", op_index, d(trace)),
            VmError::AccessMissingReturn(trace) => 
                write!(f, "Attempting to access missing return: \n{}", d(trace)),
            VmError::AccessMissingLocal(local, trace) => 
                write!(f, "Attempting to access missing local {}: \n{}", local, d(trace)),
            VmError::GenOpError(name, error, trace) => 
                write!(f, "GenOp {} encountered error {}: \n{}", name, error, d(trace)),
            VmError::TopLevelYield(ip) =>
                write!(f, "Top Level Yield no supported at instruction: {}", ip),
            VmError::AccessMissingCoroutine(coroutine, trace) =>
                write!(f, "Attempting to access missing coroutine {}: \n{}", coroutine, d(trace)),
            VmError::ResumeFinishedCoroutine(coroutine, trace) =>
                write!(f, "Attempting to resume finished coroutine {}: \n{}", coroutine, d(trace)),
        }
    }
}

impl std::error::Error for VmError { }

pub enum Slot { 
    Local(usize),
    Return,
}

pub enum Op {
    Gen(usize, Vec<Slot>),
    Call(usize, Vec<Slot>),
    ReturnSlot(Slot),
    Return,
    Branch(usize),
    DynCall(Vec<Slot>),
    Yield(Slot),
    FinishCoroutine,
    Resume(usize),
    FinishSetBranch(usize),
    // TODO  ? dup, swap, drop, move, push_from_ret
}

pub struct Fun {
    pub name : Box<str>,
    pub instrs : Vec<Op>,
}

pub struct OpEnv<'a, T, S> {
    pub locals : &'a mut Vec<Vec<T>>,
    pub globals : &'a mut Vec<S>,
    pub ret : &'a mut Option<T>,
    pub branch : &'a mut bool,
    pub dyn_call : &'a mut Option<usize>,
}

pub struct GenOp<T, S> {
    pub name : Box<str>,
    pub op : for<'a> fn(env : OpEnv<'a, T, S>, params : &Vec<Slot>) -> Result<(), Box<dyn std::error::Error>>,
}

pub struct Vm<T, S> {
    funs : Vec<Fun>,
    ops : Vec<GenOp<T, S>>,
    globals: Vec<S>,
}

struct RetAddr {
    fun : usize,
    instr : usize,
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

    // TODO append globals?

    pub fn run(&mut self, entry : usize) -> Result<Option<T>, VmError> {
        let mut fun_stack : Vec<RetAddr> = vec![];
        let mut locals : Vec<Vec<T>> = vec![]; 
        let mut ip : usize = 0;
        let mut fun : usize = entry; 
        let mut ret : Option<T> = None;
        let mut branch : bool = false;
        let mut dyn_call : Option<usize> = None;

        let mut coroutines : Vec<Vec<Coroutine<T>>> = vec![];

        // Note:  Initial locals for entry function
        locals.push(vec![]);
        coroutines.push(vec![]);
        loop {
            if fun >= self.funs.len() {
                return Err(VmError::FunDoesNotExist(fun, stack_trace(fun_stack, &self.funs)));
            }

            if ip >= self.funs[fun].instrs.len() {
                // Note:  if the current function isn't pushed onto the return stack, then the
                // stack trace will leave out the current function where the problem is occurring.
                fun_stack.push(RetAddr { fun, instr: ip });
                return Err(VmError::InstrPointerOutOfRange(ip, stack_trace(fun_stack, &self.funs)));
            }

            match self.funs[fun].instrs[ip] {
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
                            return Err(VmError::GenOpError(name, e, stack_trace(fun_stack, &self.funs))); 
                        }
                    }
                    ip += 1;
                },
                Op::Gen(op_index, _) => {
                    // Note:  Indicate current function for stack trace.
                    fun_stack.push(RetAddr { fun, instr: ip });
                    return Err(VmError::GenOpDoesNotExist(op_index, stack_trace(fun_stack, &self.funs)));
                },
                Op::Branch(target) if branch => {
                    ip = target;
                },
                Op::Branch(_) => { 
                    ip += 1;
                },
                Op::Call(fun_index, ref params) => {
                    fun_stack.push(RetAddr { fun, instr: ip + 1 });
                    fun = fun_index;
                    ip = 0;
                    let mut new_locals = vec![];
                    for param in params {
                        match get_slot(param, Cow::Borrowed(locals.last().unwrap()), Cow::Borrowed(&ret)) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                fun_stack.push(RetAddr{ fun, instr: ip });
                                return Err(f(stack_trace(fun_stack, &self.funs)));
                            },
                        }
                    }
                    locals.push(new_locals);
                    coroutines.push(vec![]);
                },
                Op::DynCall(ref params) => {
                    if dyn_call.is_none() {
                        fun_stack.push(RetAddr { fun, instr: ip });
                        return Err(VmError::DynFunDoesNotExist(stack_trace(fun_stack, &self.funs)));
                    }

                    fun_stack.push(RetAddr { fun, instr: ip + 1 });
                    fun = dyn_call.unwrap(); 
                    ip = 0;
                    let mut new_locals = vec![];
                    for param in params {
                        match get_slot(param, Cow::Borrowed(locals.last().unwrap()), Cow::Borrowed(&ret)) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                fun_stack.push(RetAddr{ fun, instr: ip });
                                return Err(f(stack_trace(fun_stack, &self.funs)));
                            },
                        }
                    }
                    locals.push(new_locals);
                    coroutines.push(vec![]);
                },
                Op::ReturnSlot(ref slot) => {
                    coroutines.pop().unwrap();
                    let current_locals = locals.pop().unwrap();

                    let ret_target = match get_slot(slot, Cow::Owned(current_locals), Cow::Owned(ret)) {
                        Ok(v) => v,
                        Err(f) => { 
                            fun_stack.push(RetAddr{ fun, instr: ip });
                            return Err(f(stack_trace(fun_stack, &self.funs)));
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
                Op::Yield(ref slot) => {
                    let current_coroutines = coroutines.pop().unwrap();
                    let current_locals = locals.pop().unwrap();
                    let current_ip = ip + 1;
                    let current_fun = fun;

                    let ret_target = match get_slot(slot, Cow::Borrowed(&current_locals), Cow::Owned(ret)) {
                        Ok(v) => v,
                        Err(f) => { 
                            fun_stack.push(RetAddr{ fun, instr: ip });
                            return Err(f(stack_trace(fun_stack, &self.funs)));
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
                Op::FinishCoroutine => {
                    match fun_stack.pop() {
                        None => {
                            // Note: Top level yields are not supported.
                            return Err(VmError::TopLevelYield(ip)); 
                        },
                        Some(ret_addr) => {
                            coroutines.pop().unwrap();
                            fun = ret_addr.fun;
                            ip = ret_addr.instr;
                            ret = None;

                            coroutines.last_mut().unwrap().push(Coroutine::Finished);
                        },
                    }
                },
                Op::Resume(coroutine) => {
                    if coroutine >= coroutines.last().unwrap().len() {
                        fun_stack.push(RetAddr{ fun, instr: ip });
                        return Err(VmError::AccessMissingCoroutine(coroutine, stack_trace(fun_stack, &self.funs)));
                    }
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
                            return Err(VmError::ResumeFinishedCoroutine(coroutine, stack_trace(fun_stack, &self.funs)))
                        },
                    }
                },
                Op::FinishSetBranch(coroutine) => {
                    if coroutine >= coroutines.last().unwrap().len() {
                        fun_stack.push(RetAddr{ fun, instr: ip });
                        return Err(VmError::AccessMissingCoroutine(coroutine, stack_trace(fun_stack, &self.funs)));
                    }

                    match coroutines.last().unwrap()[coroutine] {
                        Coroutine::Finished => { branch = true; },
                        Coroutine::Active { .. } => { branch = false; },
                    }

                },
            }
        }
    }
}

fn get_slot<T : Clone>(slot : &Slot, locals : Cow<Vec<T>>, ret : Cow<Option<T>>) 
    -> Result<T, Box<dyn Fn(StackTrace) -> VmError>> {

    match slot { 
        Slot::Return => {
            match ret {
                Cow::Borrowed(Some(v)) => Ok(v.clone()),
                Cow::Owned(Some(v)) => Ok(v),
                _ => Err(Box::new(|trace| VmError::AccessMissingReturn(trace))),
            }
        },
        Slot::Local(index) => {
            if *index >= locals.len() {
                let index = *index;
                Err(Box::new(move |trace| VmError::AccessMissingLocal(index, trace)))
            }
            else {
                match locals {
                    Cow::Borrowed(locals) => Ok(locals[*index].clone()),
                    Cow::Owned(mut locals) => Ok(locals.swap_remove(*index)),
                }
            }
        }
    }
}

fn stack_trace(stack : Vec<RetAddr>, fun_map : &[Fun]) -> StackTrace {
    let mut trace = vec![];
    for addr in stack {
        // Note:  if the function was already pushed into the stack, then
        // that means that it already resolved to a known function.  Don't
        // have to check again that the fun map has it.
        let name = fun_map[addr.fun].name.clone();
        trace.push((name, addr.instr));
    }
    trace
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_set_branch<T, S>() -> GenOp<T, S> {
        GenOp {
            name: "set".into(),
            op: |env, _| { *env.branch = true; Ok(()) },
        }
    }

    fn gen_unset_branch<T, S>() -> GenOp<T, S> {
        GenOp {
            name: "unset".into(),
            op: |env, _| { *env.branch = false; Ok(()) },
        }
    }

    fn gen_set_branch_on_zero<S>() -> GenOp<u8, S> {
        GenOp {
            name: "bz".into(),
            op: |env, params| { 
                if let [Slot::Local(s)] = &params[..] {
                    let v = env.locals.last().unwrap()[*s];
                    *env.branch = v == 0;
                }
                Ok(()) 
            },
        }
    }

    fn gen_push_global<T : Copy>() -> GenOp<T, T> {
        GenOp {
            name: "push global".into(),
            op: |env, params| { 
                if let [Slot::Local(s)] = &params[..] {
                    let v = env.globals[*s];
                    env.locals.last_mut().unwrap().push(v);
                }
                Ok(())
            },
        }
    }

    fn gen_push_into_global<T : Copy>() -> GenOp<T, T> {
        GenOp {
            name: "push into global".into(),
            op: |env, params| { 
                if let [Slot::Local(s)] = &params[..] {
                    let v = env.locals.last().unwrap()[*s];
                    env.globals.push(v);
                }
                Ok(())
            },
        }
    }

    fn gen_dec<S>() -> GenOp<u8, S> {
        GenOp {
            name: "mul".into(),
            op: | env, params |  { 
                if let [Slot::Local(s)] = &params[..] {
                    let a = &env.locals.last().unwrap()[*s];
                    *env.ret = Some(a - 1);
                }
                Ok(())
            },
        }
    }

    fn gen_mul<S>() -> GenOp<u8, S> {
        GenOp {
            name: "mul".into(),
            op: | env, params |  { 
                if let [Slot::Local(s1), Slot::Local(s2)] = &params[..] {
                    let a = &env.locals.last().unwrap()[*s1];
                    let b = &env.locals.last().unwrap()[*s2];
                    *env.ret = Some(*a * *b);
                }
                Ok(())
            },
        }
    }

    fn gen_add<S>() -> GenOp<u8, S> {
        GenOp {
            name: "add".into(),
            op: | env, params |  { 
                if let [Slot::Local(s1), Slot::Local(s2)] = &params[..] {
                    let a = &env.locals.last().unwrap()[*s1];
                    let b = &env.locals.last().unwrap()[*s2];
                    *env.ret = Some(*a + *b);
                }
                Ok(())
            },
        }
    }

    fn gen_push_return<T : Copy, S>() -> GenOp<T, S> {
        GenOp { 
            name: "push_return".into(),
            op: | env, _ | {
                let v = env.ret.unwrap();
                env.locals.last_mut().unwrap().push(v);
                Ok(())
            },
        }
    }

    #[test]
    fn should_dynamic_call() {
        const ONE : usize = 0;
        const TWO : usize = 1;
        const ADD : usize = 2;
        const PUSH_FROM_GLOBAL : usize = 3;
        const PUSH_FROM_RETURN : usize = 4;

        let add = gen_add();
        let push_from_global = gen_push_global();
        let push_from_return = gen_push_return();

        let set_dyn_call_two = GenOp { 
            name: "set two".into(),
            op: | env, _ | {
                *env.dyn_call = Some(2);
                Ok(())
            },
        };

        let set_dyn_call_one = GenOp { 
            name: "set one".into(),
            op: | env, _ | {
                *env.dyn_call = Some(1);
                Ok(())
            },
        };

        let two = Fun { 
            name: "two".into(),
            instrs: vec![
                Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(1)]), // get 2
                Op::Gen(ADD, vec![Slot::Local(0), Slot::Local(1)]),
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let one = Fun { 
            name: "one".into(),
            instrs: vec![
                Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(0)]), // get 1
                Op::Gen(ADD, vec![Slot::Local(0), Slot::Local(1)]),
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(2)]), // get 7 (now local 0)
                Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(3)]), // get 17 (now local 1)
                Op::Gen(ONE, vec![]),
                Op::DynCall(vec![Slot::Local(0)]), // should add 1 to 7 
                Op::Gen(PUSH_FROM_RETURN, vec![]), // local 2 is now 8
                Op::Gen(TWO, vec![]),
                Op::DynCall(vec![Slot::Local(1)]), // should add 2 to 17 
                Op::Gen(PUSH_FROM_RETURN, vec![]), // local 3 is now 19 
                Op::Gen(ADD, vec![Slot::Local(2), Slot::Local(3)]), // should add 19 to 8
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(
            vec![main, one, two], 
            vec![set_dyn_call_one, set_dyn_call_two, add, push_from_global, push_from_return]);

        vm.with_globals(vec![1, 2, 7, 17]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 27);
    }
    
    #[test]
    fn should_handle_multiple_calls() {
        const MUL : usize = 0;
        const PUSH_FROM_GLOBAL : usize = 1;
        const PUSH_FROM_RETURN : usize = 2;
        const BZ : usize = 3;
        const DEC : usize = 4;

        let mul = gen_mul();
        let push_from_global = gen_push_global();
        let push_from_return = gen_push_return();
        let bz = gen_set_branch_on_zero();
        let dec = gen_dec();

        let factorial = Fun { 
            name: "fact".into(),
            instrs: vec![
                Op::Gen(DEC, vec![Slot::Local(0)]),
                Op::Gen(PUSH_FROM_RETURN, vec![]),
                Op::Gen(BZ, vec![Slot::Local(1)]),
                Op::Branch(8),
                Op::Call(1, vec![Slot::Local(1)]),
                Op::Gen(PUSH_FROM_RETURN, vec![]),
                Op::Gen(MUL, vec![Slot::Local(0), Slot::Local(2)]),
                Op::ReturnSlot(Slot::Return),
                Op::ReturnSlot(Slot::Local(0)),
            ],
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(0)]),
                Op::Call(1, vec![Slot::Local(0)]),
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(
            vec![main, factorial], 
            vec![mul, push_from_global, push_from_return, bz, dec]);

        vm.with_globals(vec![5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 120);
    }

    #[test]
    fn should_return() {
        const INTO_G : usize = 0;
        const FROM_G : usize = 1;
        const ADD : usize = 2;
        const FROM_R : usize = 3;

        let push_from_global = gen_push_global();
        let push_into_global = gen_push_into_global();
        let add = gen_add();
        let push_ret = gen_push_return();

        let other = Fun { 
            name: "other".into(),
            instrs: vec![
                Op::Gen(ADD, vec![Slot::Local(0), Slot::Local(1)]),
                Op::Gen(FROM_R, vec![]),
                Op::Gen(INTO_G, vec![Slot::Local(2)]),
                Op::Return,
            ],
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Gen(FROM_G, vec![Slot::Local(1)]),
                Op::Gen(FROM_G, vec![Slot::Local(2)]),
                Op::Call(1, vec![Slot::Local(0), Slot::Local(1)]),
                Op::Gen(FROM_G, vec![Slot::Local(3)]), // from global slot 3
                Op::ReturnSlot(Slot::Local(2)), // from local slot 2
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(
            vec![main, other], 
            vec![push_into_global, push_from_global, add, push_ret]);

        vm.with_globals(vec![0, 3, 5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 8);
    }

    #[test]
    fn should_order_params() {
        let push = gen_push_global();
        let bz = gen_set_branch_on_zero();

        let other = Fun { 
            name: "other".into(),
            instrs: vec![
                Op::Gen(1, vec![Slot::Local(2)]),
                Op::Branch(3),
                Op::ReturnSlot(Slot::Local(0)),
                Op::ReturnSlot(Slot::Local(1)),
            ],
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Gen(0, vec![Slot::Local(0)]),
                Op::Gen(0, vec![Slot::Local(1)]),
                Op::Gen(0, vec![Slot::Local(2)]),
                Op::Call(1, vec![Slot::Local(2), Slot::Local(1), Slot::Local(0)]), // other(5, 3, 0)
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(
            vec![main, other], 
            vec![push, bz]);

        vm.with_globals(vec![0, 3, 5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 3);
    }
    
    #[test]
    fn should_call_with_params() {
        let push = gen_push_global();
        let add = gen_add();
        let push_ret = gen_push_return();

        let add_up = Fun { 
            name: "add_up".into(),
            instrs: vec![
                Op::Gen(1, vec![Slot::Local(0), Slot::Local(1)]),
                Op::Gen(2, vec![]),
                Op::Gen(1, vec![Slot::Local(2), Slot::Local(3)]),
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Gen(0, vec![Slot::Local(0)]),
                Op::Gen(0, vec![Slot::Local(1)]),
                Op::Gen(0, vec![Slot::Local(2)]),
                Op::Call(1, vec![Slot::Local(0), Slot::Local(1), Slot::Local(2)]),
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(
            vec![main, add_up], 
            vec![push, add, push_ret]);

        vm.with_globals(vec![2, 3, 5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 10);
    }

    #[test]
    fn should_call_and_return() {

        let push : GenOp<u8, u8> = GenOp {
            name : "push".into(),
            op: |env, _ | { 
                env.locals.last_mut().unwrap().push(9);
                Ok(())
            },
        };

        let ret_nine = Fun {
            name : "ret_nine".into(),
            instrs: vec![
                Op::Gen(0, vec![]),
                Op::ReturnSlot(Slot::Local(0)),
            ],
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Call(1, vec![]),
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(vec![main, ret_nine], vec![push]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 9);
    }

    #[test]
    fn should_branch() {
        const S : usize = 0;
        const U : usize = 1;
        const P : usize = 2;

        let set_branch: GenOp<u8, u8> = gen_set_branch();
        let unset_branch: GenOp<u8, u8> = gen_unset_branch();

        let push_stack : GenOp<u8, u8> = GenOp {
            name : "push".into(),
            op: |env, ps | { 
                if let Slot::Local(0) = ps[0] {
                    env.locals.last_mut().unwrap().push(0);
                }
                if let Slot::Local(1) = ps[0] {
                    env.locals.last_mut().unwrap().push(1);
                }
                Ok(())
            },
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Gen(S, vec![]), 
                Op::Branch(4),         
                Op::Gen(P, vec![Slot::Local(0)]),
                Op::ReturnSlot(Slot::Local(0)),

                Op::Gen(U, vec![]),
                Op::Branch(8),         
                Op::Gen(P, vec![Slot::Local(1)]),
                Op::ReturnSlot(Slot::Local(0)),

                Op::Gen(P, vec![Slot::Local(0)]),
                Op::ReturnSlot(Slot::Local(0)),
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(vec![main], vec![set_branch, unset_branch, push_stack]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 1);
    }
}
