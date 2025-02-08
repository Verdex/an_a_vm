
mod data;
mod vm;

pub enum VmError {
    X
}

pub enum Reg { 
    Frame(isize),
    Base(isize),
    Return,
}

pub enum Op {
    Gen(usize, Vec<Reg>),
    Call(usize, Vec<Reg>),
    Return(Reg),
}

pub struct Fun {
    pub name : Box<str>,
    pub instrs : Vec<Op>,
}

pub struct GenericOp<Data> {
    pub name : Box<str>,
    pub op : fn(&mut Vec<Data>, &Vec<Reg>) -> Result<(), VmError>,
}

pub struct Vm<Data> {
    fs : Vec<Fun>,
    ops : Vec<GenericOp<Data>>,
    data : Vec<Data>,
}

struct RetAddr {
    fun : usize,
    instr : usize,
    frame : usize,
}

impl<Data> Vm<Data> {
    pub fn run(&mut self, entry : usize) -> Result<Option<Data>, VmError> {
        let mut frame = self.data.len();
        let mut fun_stack : Vec<RetAddr> = vec![];
        let mut ip = 0;
        let mut current = entry;
        let mut ret : Option<Data> = None;

        loop {
            // TODO what if current does not exist
            // TODO what if ip does not exist
            match self.fs[current].instrs[ip] {
                Op::Gen(op_index, ref params) => {
                    // TODO what if op_index does not exist
                    (self.ops[op_index].op)(&mut self.data, params)?;
                    ip += 1;
                },
                Op::Call(fun_index, ref params) => {
                    fun_stack.push(RetAddr { fun: current, instr: ip + 1, frame: frame });
                    current = fun_index;
                    ip = 0;
                    frame = self.data.len();
                    // TODO move params
                },
                Op::Return(ref reg) => {
                    // TODO pop off self.data for this call, but save it off so that
                    // data can be moved to ret instead of cloned
                    // TODO what if the offset from base or frame end up outside of current local scope
                    /*let target = match reg {
                        Reg::Return => ret,
                        Reg::Base(offset) => ,
                    };*/
                },
                _ => { todo!() },
            }
        }
        Err(VmError::X)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

}
