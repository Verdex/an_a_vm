
use an_a_vm::data::*;

pub fn gen_set_branch<T, S>() -> GenOp<T, S> {
    GenOp::Frame {
        name: "set".into(),
        op: |frame, _| { frame.branch = true; Ok(None) },
    }
}

pub fn gen_unset_branch<T, S>() -> GenOp<T, S> {
    GenOp::Frame {
        name: "unset".into(),
        op: |frame, _| { frame.branch = false; Ok(None) },
    }
}

pub fn gen_set_branch_on_zero<S>() -> GenOp<u8, S> {
    GenOp::Frame {
        name: "bz".into(),
        op: |frame, params| { 
            if let [s] = &params[..] {
                let v = frame.locals[*s];
                frame.branch = v == 0;
            }
            Ok(None) 
        },
    }
}

pub fn gen_push_global<T : Copy>() -> GenOp<T, T> {
    GenOp::Vm {
        name: "push global".into(),
        op: |env, params| { 
            if let [s] = &params[..] {
                let v = env.globals[*s];
                env.current.locals.push(v);
            }
            Ok(None)
        },
    }
}

pub fn gen_push_into_global<T : Copy>() -> GenOp<T, T> {
    GenOp::Vm {
        name: "push into global".into(),
        op: |env, params| { 
            if let [s] = &params[..] {
                let v = env.current.locals[*s];
                env.globals.push(v);
            }
            Ok(None)
        },
    }
}

pub fn gen_inc<S>() -> GenOp<u8, S> {
    GenOp::Local {
        name: "inc".into(),
        op: | locals, params |  { 
            if let [s] = &params[..] {
                let a = &locals[*s];
                Ok(Some(a + 1))
            }
            else {
                Ok(None)
            }
        },
    }
}

pub fn gen_dec<S>() -> GenOp<u8, S> {
    GenOp::Local {
        name: "dec".into(),
        op: | locals, params |  { 
            if let [s] = &params[..] {
                let a = &locals[*s];
                Ok(Some(a - 1))
            }
            else {
                Ok(None)
            }
        },
    }
}

pub fn gen_mul<T : std::ops::Mul<Output = T> + Copy, S>() -> GenOp<T, S> {
    GenOp::Local {
        name: "mul".into(),
        op: | locals, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &locals[*s1];
                let b = &locals[*s2];
                Ok(Some(*a * *b))
            }
            else {
                Ok(None)
            }
        },
    }
}

pub fn gen_add<T : std::ops::Add<Output = T> + Copy, S>() -> GenOp<T, S> {
    GenOp::Local {
        name: "add".into(),
        op: | locals, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &locals[*s1];
                let b = &locals[*s2];
                Ok(Some(*a + *b))
            }
            else {
                Ok(None)
            }
        },
    }
}

pub fn gen_unset_branch_on_equal<T : PartialEq, S>() -> GenOp<T, S> {
    GenOp::Frame {
        name: "set branch on equal".into(),
        op: | frame, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &frame.locals[*s1];
                let b = &frame.locals[*s2];
                frame.branch = *a != *b;
            }
            Ok(None)
        },
    }
}

pub fn gen_set_branch_on_equal<T : PartialEq, S>() -> GenOp<T, S> {
    GenOp::Frame {
        name: "set branch on equal".into(),
        op: | frame, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &frame.locals[*s1];
                let b = &frame.locals[*s2];
                frame.branch = *a == *b;
            }
            Ok(None)
        },
    }
}

pub fn gen_set_dyn_call<S>() -> GenOp<usize, S> {
    GenOp::Frame {
        name: "set dyn call".into(),
        op: |frame, params| {
            if let [s] = &params[..] {
                let v = &frame.locals[*s];
                frame.dyn_call = Some(*v);
            }
            Ok(None)
        },
    }
}

pub fn gen_set_branch_on_finish<T, S>() -> GenOp<T, S> {
    GenOp::Frame {
        name: "set_branch_on_finish".into(),
        op: |frame, params| {
            frame.branch = !frame.coroutines[params[0]].is_alive();
            println!("BLARG {}", frame.branch);
            Ok(None)
        }
    }
}