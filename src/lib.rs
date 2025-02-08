
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
    Gen(usize),
    Call(usize, Vec<P>),
    Return(P),
}

pub struct Fun(Box<str>, Vec<Op>);

pub struct Vm<Data> {
    fs : Vec<Fun>,
    ops : Vec<(Box<str>, fn(&mut Data, Vec<P>) -> Result<(), VmError>)>,
    data : Vec<Data>,
}

impl<Data> Vm<Data> {
    pub fn run(&mut self, entry : usize) -> Result<(), VmError> {
        Err(VmError::X)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

}
