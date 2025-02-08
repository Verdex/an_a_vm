
mod data;
mod vm;

pub enum VmError {
    X
}

pub enum P {
    Frame(isize),
    Base(isize),
    Return,
}

pub enum Op {
    Gen(usize, Vec<P>),
    Call(usize, Vec<P>),
    Return(P),
}

pub struct Fun(Box<str>, Vec<Op>);

pub struct Vm<Data> {
    fs : Vec<Fun>,
    ops : Vec<(Box<str>, fn(&mut Data, Vec<P>) -> Result<(), VmError>)>,
    data : Vec<Data>,
}

struct RetAddr {
    fun : usize,
    instr_index : usize,
}

impl<Data> Vm<Data> {
    pub fn run(&mut self, entry : usize) -> Result<(), VmError> {
        let mut frame = self.data.len() - 1;
        let mut ret_stack : Option<RetAddr> = None;
        Err(VmError::X)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

}
