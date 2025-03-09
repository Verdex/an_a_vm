
use an_a_vm::data::*;

#[allow(dead_code)]
pub fn gen_set_branch<T, S>() -> GenOp<T, S> {
    GenOp {
        name: "set".into(),
        op: |env, _| { *env.branch = true; Ok(()) },
    }
}

#[allow(dead_code)]
pub fn gen_unset_branch<T, S>() -> GenOp<T, S> {
    GenOp {
        name: "unset".into(),
        op: |env, _| { *env.branch = false; Ok(()) },
    }
}

#[allow(dead_code)]
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