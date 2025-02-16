

#[derive(Debug)]
pub enum VmError {
    X
}

impl std::fmt::Display for VmError {
    fn fmt(&self, _f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { 
            // ... => write!(f, "", ...)
            _ => todo!(),
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
    pub op : fn(&mut Vec<Vec<T>>, &mut Vec<S>, &mut Option<T>, &mut bool, &Vec<Slot>) -> Result<(), VmError>,
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
        let mut ip = 0;
        let mut current = entry;
        let mut ret : Option<T> = None;
        let mut branch = false;

        // Note:  Initial locals for entry function
        self.stack.push(vec![]);
        loop {
            // TODO what if current does not exist
            // TODO what if ip does not exist
            match self.funs[current].instrs[ip] {
                Op::Gen(op_index, ref params) => {
                    // TODO what if op_index does not exist
                    (self.ops[op_index].op)(&mut self.stack, &mut self.unique, &mut ret, &mut branch, params)?;
                    ip += 1;
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
                                // TODO what if ret is none
                                new_locals.push(ret.clone().unwrap());
                            },
                            Slot::Local(index) => {
                                // TODO what if local is out of index
                                new_locals.push(self.stack[self.stack.len() - 1][*index].clone())
                            },
                        }
                    }
                    self.stack.push(new_locals);
                },
                Op::ReturnSlot(ref slot) => {
                    let mut current_locals = self.stack.pop().unwrap();

                    let ret_target = match slot {
                        Slot::Local(index) => current_locals.swap_remove(*index), // TODO what if this isn't something
                        Slot::Return => ret.unwrap(), // TODO what if this isn't something
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
