

// TODO Rc instead of Box (for names)
// TODO Rc instead of Vec?  (surely not in all instances)

// TODO does frame allow lambda impl?

use std::rc::Rc;

pub enum Op<T> {
    Gen(usize, Vec<usize>),
    Call(usize, Vec<usize>),
    ReturnLocal(usize), 
    Return,
    Branch(usize),
    DynCall(Vec<usize>),
    Drop(usize),
    Dup(usize),
    Swap(usize, usize),
    PushRet,
    PushLocal(T),
    CoYield(usize),
    CoFinish,
    CoResume(usize),
    // TODO now with CoDrop this op doesn't need to delete the coroutine
    // TODO potentially this op doesn't need to exist
    CoFinishSetBranch(usize),
    CoDrop(usize),
    CoDup(usize), 
    CoSwap(usize, usize),
}

pub struct Fun<T> {
    pub name : Box<str>,
    pub instrs : Vec<Op<T>>,
}

pub struct VmEnv<'a, T, S> {
    pub globals: &'a mut Vec<S>,
    pub frames : &'a mut Vec<Frame<T>>,
    pub current : &'a mut Frame<T>,
}

pub enum GenOp<T, S> {
    Vm { name : Rc<str>, op : for<'a> fn(vm : VmEnv<'a, T, S>, params : &[usize]) -> Result<Option<T>, Box<dyn std::error::Error>> },
    Global { name : Rc<str>, op : fn(globals : &mut Vec<S>, params : &[usize]) -> Result<Option<T>, Box<dyn std::error::Error>> },
    Local { name : Rc<str>, op : fn(locals : &mut Vec<T>, params : &[usize]) -> Result<Option<T>, Box<dyn std::error::Error>> },
    Frame { name : Rc<str>, op : fn(frame : &mut Frame<T>, params : &[usize]) -> Result<Option<T>, Box<dyn std::error::Error>> },
}

#[derive(Clone)]
pub struct Frame<T> {
    pub (crate) fun_id : usize,
    pub (crate) ip : usize,
    pub (crate) ret : Option<T>,
    pub branch : bool,
    pub dyn_call : Option<usize>,
    pub locals : Vec<T>,
    pub coroutines : Vec<Coroutine<T>>,
}

#[derive(Clone)]
pub enum Coroutine<T> {
    Active(Frame<T>),
    Running,
    Finished,
}
