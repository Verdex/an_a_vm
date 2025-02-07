use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum LAddr {
    Local(usize),
    Env(usize),
}

#[derive(Debug, Clone)]
pub enum Lit {
    Float(f64),
    Ref(LAddr),
    Unit,
}

#[derive(Debug)]
pub enum Stmt {
    Deref(LAddr, usize),
    Add(LAddr, LAddr),
    Cons(Vec<Lit>),
    Return(LAddr),
    Call(Rc<Fun>, Vec<LAddr>),
    DPrint(Vec<LAddr>),
}

#[derive(Debug)]
pub struct Fun { 
    pub name : Box<str>,
    pub body : Vec<Stmt>,
}