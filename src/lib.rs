

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

pub struct GenericOp<Data, Unique> {
    pub name : Box<str>,
    pub op : fn(&mut Vec<Vec<Data>>, &mut Vec<Unique>, &mut Option<Data>, &mut bool, &Vec<Slot>) -> Result<(), VmError>,
}

pub struct Vm<Data, Unique> {
    pub fs : Vec<Fun>,
    pub ops : Vec<GenericOp<Data, Unique>>,
    pub data : Vec<Vec<Data>>,
    pub unique : Vec<Unique>,
}

struct RetAddr {
    fun : usize,
    instr : usize,
}

impl<Data : Clone, Unique> Vm<Data, Unique> {
    pub fn run(&mut self, entry : usize) -> Result<Option<Data>, VmError> {
        let mut fun_stack : Vec<RetAddr> = vec![];
        let mut ip = 0;
        let mut current = entry;
        let mut ret : Option<Data> = None;
        let mut branch = false;

        // Note:  Initial locals for entry function
        self.data.push(vec![]);
        loop {
            // TODO what if current does not exist
            // TODO what if ip does not exist
            match self.fs[current].instrs[ip] {
                Op::Gen(op_index, ref params) => {
                    // TODO what if op_index does not exist
                    (self.ops[op_index].op)(&mut self.data, &mut self.unique, &mut ret, &mut branch, params)?;
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
                                new_locals.push(self.data[self.data.len() - 1][*index].clone())
                            },
                        }
                    }
                    self.data.push(new_locals);
                },
                Op::ReturnSlot(ref slot) => {
                    let mut current_locals = self.data.pop().unwrap();

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
                            self.data.pop();
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

}
