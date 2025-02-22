

pub type StackTrace = Vec<(Box<str>, usize)>;

#[derive(Debug)]
pub enum VmError {
    FunDoesNotExist(usize, StackTrace),
    InstrPointerOutOfRange(usize, StackTrace),
    GenOpDoesNotExist(usize, StackTrace),
    CallAccessMissingReturn(StackTrace),
    CallAccessMissingLocal(usize, StackTrace),
    ReturnAccessMissingReturn(StackTrace),
    ReturnAccessMissingLocal(usize, StackTrace),
    GenOpError(Box<str>, Box<dyn std::error::Error>, StackTrace),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        fn d(x : &StackTrace) -> String {
            x.into_iter().map(|(n, i)| format!("    {} at index {}\n", n, i)).collect()
        }

        match self { 
            VmError::FunDoesNotExist(fun_index, trace) => 
                write!(f, "Fun Index {} does not exist: \n{}", fun_index, d(trace)),
            VmError::InstrPointerOutOfRange(instr, trace) => 
                write!(f, "Instr Index {} does not exist: \n{}", instr, d(trace)),
            VmError::GenOpDoesNotExist(op_index, trace) => 
                write!(f, "GenOp {} does not exist: \n{}", op_index, d(trace)),
            VmError::CallAccessMissingReturn(trace) => 
                write!(f, "Call attempting to access missing return: \n{}", d(trace)),
            VmError::CallAccessMissingLocal(local, trace) => 
                write!(f, "Call attempting to access missing local {}: \n{}", local, d(trace)),
            VmError::ReturnAccessMissingReturn(trace) => 
                write!(f, "Return attempting to access missing return: \n{}", d(trace)),
            VmError::ReturnAccessMissingLocal(local, trace) => 
                write!(f, "Return attempting to access missing local {}: \n{}", local, d(trace)),
            VmError::GenOpError(name, error, trace) => 
                write!(f, "GenOp {} encountered error {}: \n{}", name, error, d(trace)),
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
}

pub struct Fun {
    pub name : Box<str>,
    pub instrs : Vec<Op>,
}

pub struct GenOp<T, S> {
    pub name : Box<str>,
    pub op : fn(&mut Vec<Vec<T>>, &mut Vec<S>, &mut Option<T>, &mut bool, &Vec<Slot>) -> Result<(), Box<dyn std::error::Error>>,
}

pub struct Vm<T, S> {
    funs : Vec<Fun>,
    ops : Vec<GenOp<T, S>>,
    stack : Vec<Vec<T>>,
    unique : Vec<S>,
}

struct RetAddr {
    fun : usize,
    instr : usize,
}

impl<T : Clone, S> Vm<T, S> {
    pub fn new(funs : Vec<Fun>, ops : Vec<GenOp<T, S>>) -> Self {
        Vm { funs, ops, stack: vec![], unique: vec![] }
    }

    pub fn with_unique(&mut self, unique : Vec<S>) -> Vec<S> {
        std::mem::replace(&mut self.unique, unique)
    }

    pub fn run(&mut self, entry : usize) -> Result<Option<T>, VmError> {
        let mut fun_stack : Vec<RetAddr> = vec![];
        let mut ip : usize = 0;
        let mut current : usize = entry;
        let mut ret : Option<T> = None;
        let mut branch : bool = false;

        // Note:  Initial locals for entry function
        self.stack.push(vec![]);
        loop {
            if current >= self.funs.len() {
                return Err(VmError::FunDoesNotExist(current, stack_trace(&fun_stack, &self.funs)));
            }

            if ip >= self.funs[current].instrs.len() {
                // Note:  if the current function isn't pushed onto the return stack, then the
                // stack trace will leave out the current function where the problem is occurring.
                fun_stack.push(RetAddr { fun: current, instr: ip });
                return Err(VmError::InstrPointerOutOfRange(ip, stack_trace(&fun_stack, &self.funs)));
            }

            match self.funs[current].instrs[ip] {
                Op::Gen(op_index, ref params) if op_index < self.ops.len() => {
                    match (self.ops[op_index].op)(&mut self.stack, &mut self.unique, &mut ret, &mut branch, params) {
                        Ok(()) => { },
                        Err(e) => { 
                            let name = self.ops[op_index].name.clone();
                            fun_stack.push(RetAddr { fun: current, instr: ip });
                            return Err(VmError::GenOpError(name, e, stack_trace(&fun_stack, &self.funs))); 
                        }
                    }
                    ip += 1;
                },
                Op::Gen(op_index, _) => {
                    // Note:  Indicate current function for stack trace.
                    fun_stack.push(RetAddr { fun: current, instr: ip });
                    return Err(VmError::GenOpDoesNotExist(op_index, stack_trace(&fun_stack, &self.funs)));
                },
                Op::Branch(target) if branch => {
                    ip = target;
                },
                Op::Branch(_) => { 
                    ip += 1;
                },
                Op::Call(fun_index, ref params) => {
                    fun_stack.push(RetAddr { fun: current, instr: ip + 1 });
                    current = fun_index;
                    ip = 0;
                    let mut new_locals = vec![];
                    for param in params {
                        match param { 
                            Slot::Return => {
                                match ret {
                                    Some(ref v) => { new_locals.push(v.clone()); },
                                    None => {
                                        fun_stack.push(RetAddr{ fun: current, instr: ip });
                                        return Err(VmError::CallAccessMissingReturn(stack_trace(&fun_stack, &self.funs)));
                                    },
                                }
                            },
                            Slot::Local(index) => {
                                if *index >= self.stack[self.stack.len() - 1].len() {
                                    fun_stack.push(RetAddr{ fun: current, instr: ip });
                                    return Err(VmError::CallAccessMissingLocal(*index, stack_trace(&fun_stack, &self.funs)));
                                }

                                new_locals.push(self.stack[self.stack.len() - 1][*index].clone())
                            },
                        }
                    }
                    self.stack.push(new_locals);
                },
                Op::ReturnSlot(ref slot) => {
                    let mut current_locals = self.stack.pop().unwrap();

                    let ret_target = match slot {
                        Slot::Local(index) => {
                            if *index >= current_locals.len() {
                                fun_stack.push(RetAddr{ fun: current, instr: ip });
                                return Err(VmError::ReturnAccessMissingLocal(*index, stack_trace(&fun_stack, &self.funs)));
                            }

                            current_locals.swap_remove(*index)
                        }, 
                        Slot::Return => {
                            match ret {
                                Some(v) => v,
                                None => {
                                    fun_stack.push(RetAddr{ fun: current, instr: ip });
                                    return Err(VmError::ReturnAccessMissingReturn(stack_trace(&fun_stack, &self.funs)));
                                },
                            }
                        }, 
                    };

                    match fun_stack.pop() {
                        // Note:  if the stack is empty then all execution is finished
                        None => {
                            return Ok(Some(ret_target));
                        },
                        Some(ret_addr) => {
                            current = ret_addr.fun;
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
                            self.stack.pop();
                            current = ret_addr.fun;
                            ip = ret_addr.instr;
                            ret = None;
                        },
                    }
                },
            }
        }
    }
}

fn stack_trace(stack : &[RetAddr], fun_map : &[Fun]) -> StackTrace {
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
            op: |_, _, _, b, _| { *b = true; Ok(()) },
        }
    }

    fn gen_unset_branch<T, S>() -> GenOp<T, S> {
        GenOp {
            name: "unset".into(),
            op: |_, _, _, b, _| { *b = false; Ok(()) },
        }
    }

    fn gen_set_branch_on_zero<S>() -> GenOp<u8, S> {
        GenOp {
            name: "bz".into(),
            op: |d, _, _, b, params| { 
                if let [Slot::Local(s)] = &params[..] {
                    let v = d.last().unwrap()[*s];
                    *b = v == 0;
                }
                Ok(()) 
            },
        }
    }

    fn gen_push_unique<T : Copy>() -> GenOp<T, T> {
        GenOp {
            name: "push unique".into(),
            op: |d, u, _, _, params| { 
                if let [Slot::Local(s)] = &params[..] {
                    let v = u[*s];
                    d.last_mut().unwrap().push(v);
                }
                Ok(())
            },
        }
    }

    fn gen_push_into_unique<T : Copy>() -> GenOp<T, T> {
        GenOp {
            name: "push into unique".into(),
            op: |d, u, _, _, params| { 
                if let [Slot::Local(s)] = &params[..] {
                    let v = d.last().unwrap()[*s];
                    u.push(v);
                }
                Ok(())
            },
        }
    }

    fn gen_dec<S>() -> GenOp<u8, S> {
        GenOp {
            name: "mul".into(),
            op: | d, _, ret, _, params |  { 
                if let [Slot::Local(s)] = &params[..] {
                    let a = &d.last().unwrap()[*s];
                    *ret = Some(a - 1);
                }
                Ok(())
            },
        }
    }

    fn gen_mul<S>() -> GenOp<u8, S> {
        GenOp {
            name: "mul".into(),
            op: | d, _, ret, _, params |  { 
                if let [Slot::Local(s1), Slot::Local(s2)] = &params[..] {
                    let a = &d.last().unwrap()[*s1];
                    let b = &d.last().unwrap()[*s2];
                    *ret = Some(*a * *b);
                }
                Ok(())
            },
        }
    }

    fn gen_add<S>() -> GenOp<u8, S> {
        GenOp {
            name: "add".into(),
            op: | d, _, ret, _, params |  { 
                if let [Slot::Local(s1), Slot::Local(s2)] = &params[..] {
                    let a = &d.last().unwrap()[*s1];
                    let b = &d.last().unwrap()[*s2];
                    *ret = Some(*a + *b);
                }
                Ok(())
            },
        }
    }

    fn gen_push_return<T : Copy, S>() -> GenOp<T, S> {
        GenOp { 
            name: "push_return".into(),
            op: | d, _, ret, _, _ | {
                let v = ret.unwrap();
                d.last_mut().unwrap().push(v);
                Ok(())
            },
        }
    }
    
    #[test]
    fn should_handle_multiple_calls() {
        const MUL : usize = 0;
        const PUSH_FROM_UNIQUE : usize = 1;
        const PUSH_FROM_RETURN : usize = 2;
        const BZ : usize = 3;
        const DEC : usize = 4;

        let mul = gen_mul();
        let push_from_unique = gen_push_unique();
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
                Op::Gen(PUSH_FROM_UNIQUE, vec![Slot::Local(0)]),
                Op::Call(1, vec![Slot::Local(0)]),
                Op::ReturnSlot(Slot::Return),
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(
            vec![main, factorial], 
            vec![mul, push_from_unique, push_from_return, bz, dec]);

        vm.with_unique(vec![5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 120);
    }

    #[test]
    fn should_return() {
        const INTO_U : usize = 0;
        const FROM_U : usize = 1;
        const ADD : usize = 2;
        const FROM_R : usize = 3;

        let push_from_unique = gen_push_unique();
        let push_into_unique = gen_push_into_unique();
        let add = gen_add();
        let push_ret = gen_push_return();

        let other = Fun { 
            name: "other".into(),
            instrs: vec![
                Op::Gen(ADD, vec![Slot::Local(0), Slot::Local(1)]),
                Op::Gen(FROM_R, vec![]),
                Op::Gen(INTO_U, vec![Slot::Local(2)]),
                Op::Return,
            ],
        };

        let main = Fun { 
            name: "main".into(),
            instrs: vec![
                Op::Gen(FROM_U, vec![Slot::Local(1)]),
                Op::Gen(FROM_U, vec![Slot::Local(2)]),
                Op::Call(1, vec![Slot::Local(0), Slot::Local(1)]),
                Op::Gen(FROM_U, vec![Slot::Local(3)]), // from unique slot 3
                Op::ReturnSlot(Slot::Local(2)), // from stack slot 2
            ],
        };

        let mut vm : Vm<u8, u8> = Vm::new(
            vec![main, other], 
            vec![push_into_unique, push_from_unique, add, push_ret]);

        vm.with_unique(vec![0, 3, 5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 8);
    }

    #[test]
    fn should_order_params() {
        let push = gen_push_unique();
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

        vm.with_unique(vec![0, 3, 5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 3);
    }
    
    #[test]
    fn should_call_with_params() {
        let push = gen_push_unique();
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

        vm.with_unique(vec![2, 3, 5]);

        let data = vm.run(0).unwrap().unwrap();

        assert_eq!(data, 10);
    }

    #[test]
    fn should_call_and_return() {

        let push : GenOp<u8, u8> = GenOp {
            name : "push".into(),
            op: |d, _, _, _, _ | { 
                let l = d.len() - 1;
                d[l].push(9);
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
            op: |d, _, _, _, ps | { 
                let l = d.len() - 1;
                if let Slot::Local(0) = ps[0] {
                    d[l].push(0);
                }
                if let Slot::Local(1) = ps[0] {
                    d[l].push(1);
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
