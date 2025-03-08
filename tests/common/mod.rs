
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
            if let [Slot::Local(s)] = &params[..] {
                let v = env.locals.last().unwrap()[*s];
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
            if let [Slot::Local(s)] = &params[..] {
                let v = env.globals[*s];
                env.locals.last_mut().unwrap().push(v);
            }
            Ok(())
        },
    }
}

pub fn gen_push_into_global<T : Copy>() -> GenOp<T, T> {
    GenOp {
        name: "push into global".into(),
        op: |env, params| { 
            if let [Slot::Local(s)] = &params[..] {
                let v = env.locals.last().unwrap()[*s];
                env.globals.push(v);
            }
            Ok(())
        },
    }
}

pub fn gen_dec<S>() -> GenOp<u8, S> {
    GenOp {
        name: "mul".into(),
        op: | env, params |  { 
            if let [Slot::Local(s)] = &params[..] {
                let a = &env.locals.last().unwrap()[*s];
                *env.ret = Some(a - 1);
            }
            Ok(())
        },
    }
}

pub fn gen_mul<S>() -> GenOp<u8, S> {
    GenOp {
        name: "mul".into(),
        op: | env, params |  { 
            if let [Slot::Local(s1), Slot::Local(s2)] = &params[..] {
                let a = &env.locals.last().unwrap()[*s1];
                let b = &env.locals.last().unwrap()[*s2];
                *env.ret = Some(*a * *b);
            }
            Ok(())
        },
    }
}

pub fn gen_add<S>() -> GenOp<u8, S> {
    GenOp {
        name: "add".into(),
        op: | env, params |  { 
            if let [Slot::Local(s1), Slot::Local(s2)] = &params[..] {
                let a = &env.locals.last().unwrap()[*s1];
                let b = &env.locals.last().unwrap()[*s2];
                *env.ret = Some(*a + *b);
            }
            Ok(())
        },
    }
}
