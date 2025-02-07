use std::rc::Rc;

use crate::data::*;

#[derive(Debug, Clone, Copy)]
pub struct HAddr(usize);

#[derive(Debug, Clone)]
pub enum Val {
    Float(f64),
    Ref(HAddr),
    Unit,
}

impl Val {
    pub fn float(&self) -> f64 {
        match self {
            Val::Float(x) => *x,
            x => panic!("Expected Float but found {:?}", x),
        }
    }
    pub fn reference(&self) -> HAddr {
        match self {
            Val::Ref(x) => *x,
            x => panic!("Expected Ref but found {:?}", x),
        }
    }
}

#[derive(Debug)]
struct Frame { 
    ip : usize,
    locals : Vec<HAddr>,
    fun : Rc<Fun>,
}

#[derive(Debug)]
pub struct Vm {
    heap : Vec<Val>,
    stack : Vec<Frame>,
}

// TODO:  le patterns probably need:  generators, foreach, seq, and anon-structs (or row poly or pattern env types)

impl Vm {
    pub fn new() -> Self { 
        Vm { heap : vec![] 
           , stack : vec![]
           }
    }

    pub fn run(&mut self, main : Rc<Fun>, env : &[HAddr]) -> HAddr {
        run_vm(self, main, env)
    }

    pub fn get_val(&mut self, addr : HAddr) -> &mut Val {
        self.heap.sget(addr)
    }
}

trait Heap { 
    fn sget(&mut self, index : HAddr) -> &mut Val;
    fn cons_vals(&mut self, v : Vec<Val>) -> HAddr;
}

impl Heap for Vec<Val> {
    fn sget(&mut self, index : HAddr) -> &mut Val { 
        self.get_mut(index.0).unwrap()
    }
    fn cons_vals(&mut self, mut vs : Vec<Val>) -> HAddr {
        let addr = self.len();
        self.append(&mut vs);
        HAddr(addr)
    }
}

// Assume:  every body has a return at the end
fn run_vm(m : &mut Vm, main : Rc<Fun>, env : &[HAddr]) -> HAddr {
    let mut ip : usize = 0;
    let mut locals : Vec<HAddr> = vec![];
    let mut current : Rc<Fun> = main;
    loop {
        match current.body[ip] {
            Stmt::Deref(ref addr, offset) => {
                let r = m.heap.sget(local_lookup(addr, &locals, env)).reference();
                locals.push(HAddr(r.0 + offset));
                ip += 1;
            },
            Stmt::Add(ref a, ref b) => {
                let a = m.heap.sget(local_lookup(a, &locals, env)).float();
                let b = m.heap.sget(local_lookup(b, &locals, env)).float();
                let addr = m.heap.cons_vals(vec![Val::Float(a + b)]);
                locals.push(addr);
                ip += 1;
            },
            Stmt::Cons(ref ls) => {
                let addr = m.heap.cons_vals(ls.iter().map(|x| lit_to_val(x, &locals, env)).collect());
                locals.push(addr);
                ip += 1;
            },
            Stmt::Return(ref addr) => {
                let ret = local_lookup(addr, &locals, env);
                if let Some(frame) = m.stack.pop() {
                    ip = frame.ip;
                    locals = frame.locals;
                    current = frame.fun;

                    locals.push(ret);
                }
                else {
                    return ret;
                }
            },
            Stmt::Call(ref new, ref params) => {
                let new_locals = params.iter().map(|x| local_lookup(x, &locals, env)).collect::<Vec<_>>();
                let ls = std::mem::replace(&mut locals, new_locals);
                m.stack.push(Frame { fun: Rc::clone(&current), ip: ip + 1, locals: ls });
                ip = 0;
                current = Rc::clone(new);
            },
            Stmt::DPrint(ref params) => {
                let targets = params.iter()
                                    .map(|x| format!("{:?}", m.heap.sget(local_lookup(x, &locals, env))))
                                    .collect::<Vec<_>>();
                println!("{:?}", targets);
                ip += 1;
            },
            _ => todo!(),
        }
    }
}

fn lit_to_val(lit : &Lit, locals : &Vec<HAddr>, env : &[HAddr]) -> Val {
    match lit {
        Lit::Float(f) => Val::Float(*f),
        Lit::Unit => Val::Unit,
        Lit::Ref(laddr) => Val::Ref(local_lookup(laddr, locals, env)),
    }
}

fn local_lookup(addr : &LAddr, locals : &Vec<HAddr>, env : &[HAddr]) -> HAddr {
    match addr {
        LAddr::Local(x) => locals[*x],
        LAddr::Env(x) => env[*x],
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! proj {
        ($input:expr, $p:pat, $e:expr) => { 
            match $input {
                $p => $e,
                _ => unreachable!(),
            }
        }
    }

    fn local(x : usize) -> LAddr {
        LAddr::Local(x)
    }

    fn env(x : usize) -> LAddr {
        LAddr::Env(x)
    }

    #[test]
    fn should_call() {
        let mut vm = Vm::new();

        let adder_body = 
            vec![ Stmt::Add(local(0), local(1))
                , Stmt::Return(local(2)) 
                ];
        
        let adder : Rc<Fun> = Fun { name: "adder".into()
                                  , body: adder_body
                                  }.into();

        let main_body = 
            vec![ Stmt::Cons(vec![Lit::Float(1.0)])
                , Stmt::Cons(vec![Lit::Float(2.0)])
                , Stmt::Call(adder, vec![local(0), local(1)])
                , Stmt::Return(local(2)) 
                ];

        let main = Fun { name: "main".into()
                       , body: main_body 
                       };

        let ret = vm.run(main.into(), &[]);

        let v = vm.get_val(ret);
        assert_eq!(proj!(v, Val::Float(x), *x), 3.0);
    }

    #[test]
    fn should_add_env_with_local() {
        let mut vm = Vm::new();

        let body = 
            vec![ Stmt::Cons(vec![Lit::Float(1.0)])
                , Stmt::Return(local(0)) 
                ];

        let f = Fun { name: "x".into()
                    , body
                    };

        let ret = vm.run(f.into(), &[]);

        let body = vec![ Stmt::Cons(vec![Lit::Float(2.0)])
                       , Stmt::Add(env(0), local(0))
                       , Stmt::Return(local(1))
                       ];

        let f = Fun { name: "x".into()
                    , body
                    };

        let ret = vm.run(f.into(), &[ret]);
        
        let v = vm.get_val(ret);
        assert_eq!(proj!(v, Val::Float(x), *x), 3.0);
    }

    #[test]
    fn should_return_val_ref() {
        let mut vm = Vm::new();

        let body = 
            vec![ Stmt::Cons(vec![Lit::Float(1.0)])
                , Stmt::Return(local(0)) 
                ];

        let f = Fun { name: "x".into()
                    , body
                    };

        let ret = vm.run(f.into(), &[]);

        let v = vm.get_val(ret);
        assert_eq!(proj!(v, Val::Float(x), *x), 1.0);
    }

    #[test]
    fn should_return_env_ref() {
        let mut vm = Vm::new();

        let body = 
            vec![ Stmt::Cons(vec![Lit::Float(1.0)])
                , Stmt::Return(local(0)) 
                ];

        let f = Fun { name: "x".into()
                    , body
                    };

        let ret = vm.run(f.into(), &[]);

        let body = vec![Stmt::Return(env(0))];

        let f = Fun { name: "x".into()
                    , body
                    };

        let ret = vm.run(f.into(), &[ret]);
        
        let v = vm.get_val(ret);
        assert_eq!(proj!(v, Val::Float(x), *x), 1.0);
    }

    #[test]
    fn should_cons_locals() {
        let mut vm = Vm::new();

        let body = 
            vec![ Stmt::Cons(vec![Lit::Float(1.0)])
                , Stmt::Cons(vec![Lit::Float(2.0)])
                , Stmt::Cons(vec![Lit::Ref(local(0)), Lit::Ref(local(1))])
                , Stmt::Deref(local(2), 0)
                , Stmt::Deref(local(2), 1)
                , Stmt::Add(local(3), local(4))
                , Stmt::Return(local(5)) 
                ];

        let f = Fun { name: "x".into()
                    , body
                    };

        let ret = vm.run(f.into(), &[]);

        let v = vm.get_val(ret);
        assert_eq!(proj!(v, Val::Float(x), *x), 3.0);
    }
}