
use an_a_vm::data::*;

pub fn gen_set_branch<T, S>() -> GenOp<T, S> {
    GenOp {
        name: "set".into(),
        op: |env, _| { *env.branch = true; Ok(()) },
    }
}

pub fn gen_unset_branch<T, S>() -> GenOp<T, S> {
    GenOp {
        name: "unset".into(),
        op: |env, _| { *env.branch = false; Ok(()) },
    }
}

pub fn gen_set_branch_on_zero<S>() -> GenOp<u8, S> {
    GenOp {
        name: "bz".into(),
        op: |env, params| { 
            if let [s] = &params[..] {
                let v = env.locals[*s];
                *env.branch = v == 0;
            }
            Ok(()) 
        },
    }
}

pub fn gen_push_global<T : Copy>() -> GenOp<T, T> {
    GenOp {
        name: "push global".into(),
        op: |env, params| { 
            if let [s] = &params[..] {
                let v = env.globals[*s];
                env.locals.push(v);
            }
            Ok(())
        },
    }
}

pub fn gen_push_into_global<T : Copy>() -> GenOp<T, T> {
    GenOp {
        name: "push into global".into(),
        op: |env, params| { 
            if let [s] = &params[..] {
                let v = env.locals[*s];
                env.globals.push(v);
            }
            Ok(())
        },
    }
}

pub fn gen_inc<S>() -> GenOp<u8, S> {
    GenOp {
        name: "inc".into(),
        op: | env, params |  { 
            if let [s] = &params[..] {
                let a = &env.locals[*s];
                *env.ret = Some(a + 1);
            }
            Ok(())
        },
    }
}

pub fn gen_dec<S>() -> GenOp<u8, S> {
    GenOp {
        name: "dec".into(),
        op: | env, params |  { 
            if let [s] = &params[..] {
                let a = &env.locals[*s];
                *env.ret = Some(a - 1);
            }
            Ok(())
        },
    }
}

pub fn gen_mul<T : std::ops::Mul<Output = T> + Copy, S>() -> GenOp<T, S> {
    GenOp {
        name: "mul".into(),
        op: | env, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &env.locals[*s1];
                let b = &env.locals[*s2];
                *env.ret = Some(*a * *b);
            }
            Ok(())
        },
    }
}

pub fn gen_add<T : std::ops::Add<Output = T> + Copy, S>() -> GenOp<T, S> {
    GenOp {
        name: "add".into(),
        op: | env, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &env.locals[*s1];
                let b = &env.locals[*s2];
                *env.ret = Some(*a + *b);
            }
            Ok(())
        },
    }
}

pub fn gen_unset_branch_on_equal<T : PartialEq, S>() -> GenOp<T, S> {
    GenOp {
        name: "set branch on equal".into(),
        op: | env, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &env.locals[*s1];
                let b = &env.locals[*s2];
                *env.branch = *a != *b;
            }
            Ok(())
        },
    }
}

pub fn gen_set_branch_on_equal<T : PartialEq, S>() -> GenOp<T, S> {
    GenOp {
        name: "set branch on equal".into(),
        op: | env, params |  { 
            if let [s1, s2] = &params[..] {
                let a = &env.locals[*s1];
                let b = &env.locals[*s2];
                *env.branch = *a == *b;
            }
            Ok(())
        },
    }
}

pub fn gen_set_dyn_call<S>() -> GenOp<usize, S> {
    GenOp {
        name: "set dyn call".into(),
        op: |env, params| {
            if let [s] = &params[..] {
                let v = &env.locals[*s];
                *env.dyn_call = Some(*v);
            }
            Ok(())
        },
    }
}
