
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

pub enum Op<Data> {
    Gen(Box<str>, fn(&mut Data, Vec<P>) -> Result<(), VmError>),
    Call(usize, Vec<P>),
    Return(P),
}

pub struct Fun<Data>(Box<str>, Vec<Op<Data>>);

pub struct Vm<Data> {
    fs : Vec<Fun<Data>>,
    ops : Vec<Op<Data>>,
    data : Vec<Data>,
}

impl<Data> Vm<Data> {
    pub fn run(&mut self) -> Result<(), VmError> {
        Err(VmError::X)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

}
