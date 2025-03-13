
pub enum Slot { 
    Local(usize),
    Return,
}

pub enum Op {
    Gen(usize, Vec<usize>),
    Call(usize, Vec<Slot>),
    ReturnSlot(Slot),
    Return,
    Branch(usize),
    DynCall(Vec<Slot>),
    Yield(Slot),
    Finish,
    Resume(usize),
    FinishSetBranch(usize),
    Drop(usize),
    Dup(usize),
    Swap(usize, usize),
    PushRet,
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
    pub op : for<'a> fn(env : OpEnv<'a, T, S>, params : &Vec<usize>) -> Result<(), Box<dyn std::error::Error>>,
}
