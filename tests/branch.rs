
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;


#[test]
fn should_branch() {
    const S : usize = 0;
    const U : usize = 1;
    const P : usize = 2;

    let set_branch: GenOp<u8, u8> = common::gen_set_branch();
    let unset_branch: GenOp<u8, u8> = common::gen_unset_branch();

    let push_stack : GenOp<u8, u8> = GenOp {
        name : "push".into(),
        op: |env, ps | { 
            if ps[0] == 0 {
                env.locals.last_mut().unwrap().push(0);
            }
            if ps[0] == 1 {
                env.locals.last_mut().unwrap().push(1);
            }
            Ok(())
        },
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(S, vec![]), 
            Op::Branch(4),         
            Op::Gen(P, vec![0]),
            Op::ReturnSlot(Slot::Local(0)),

            Op::Gen(U, vec![]),
            Op::Branch(8),         
            Op::Gen(P, vec![1]),
            Op::ReturnSlot(Slot::Local(0)),

            Op::Gen(P, vec![0]),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(vec![main], vec![set_branch, unset_branch, push_stack]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 1);
}

#[test]
fn should_loop() {
    const INC : usize = 0;
    const DEC : usize = 1;
    const PUSH : usize = 2;
    const SBE : usize = 3;
    const SET : usize = 4;

    let inc = common::gen_inc();
    let dec = common::gen_dec();
    let push_from_global = common::gen_push_global();
    let set_branch_on_equal = common::gen_set_branch_on_equal();
    let set_branch = common::gen_set_branch();

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(PUSH, vec![0]),
            Op::Gen(PUSH, vec![0]),
            Op::Gen(PUSH, vec![1]),
            Op::Gen(SBE, vec![0, 2]),
            Op::Branch(19), 
            Op::Gen(INC, vec![1]),
            Op::PushRet,
            Op::Swap(1, 3),
            Op::Drop(3),
            Op::Gen(INC, vec![1]),
            Op::PushRet,
            Op::Swap(1, 3),
            Op::Drop(3),
            Op::Gen(DEC, vec![2]),
            Op::PushRet,
            Op::Swap(2, 3),
            Op::Drop(3),
            Op::Gen(SET, vec![]),
            Op::Branch(3), 
            Op::ReturnSlot(Slot::Local(1)),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main], 
        vec![inc, dec, push_from_global, set_branch_on_equal, set_branch]);

    vm.with_globals(vec![0, 10]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 20);
}

#[test]
fn should_not_branch_on_active_coroutine() {
    let push_from_global = common::gen_push_global();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Yield(Slot::Local(0)),
            Op::Finish,
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::FinishSetBranch(0),
            Op::Branch(4),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, co], 
        vec![push_from_global]);

    vm.with_globals(vec![1, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_branch_on_finished_coroutine() {
    let push_from_global = common::gen_push_global();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Finish,
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::FinishSetBranch(0),
            Op::Branch(4),
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, co], 
        vec![push_from_global]);

    vm.with_globals(vec![1, 3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_branch_on_finished_coroutine_with_active_coroutine_present() {
    let push_from_global = common::gen_push_global();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Yield(Slot::Local(0)),
            Op::Finish,
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::Call(1, vec![]),
            Op::Resume(1),
            Op::FinishSetBranch(1),
            Op::Branch(6),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, co], 
        vec![push_from_global]);

    vm.with_globals(vec![1, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 5);
}

#[test]
fn should_branch_on_finished_coroutine_in_function_where_parent_has_active_coroutine() {
    let push_from_global = common::gen_push_global();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Yield(Slot::Local(0)),
            Op::Finish,
        ],
    };

    let child = Fun {
        name: "child".into(),
        instrs: vec![
            Op::Call(2, vec![]),
            Op::Resume(0),
            Op::FinishSetBranch(0),
            Op::Branch(5),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(2, vec![]),
            Op::Call(1, vec![]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, child, co], 
        vec![push_from_global]);

    vm.with_globals(vec![1, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 5);
}

#[test]
fn should_branch_on_finished_coroutine_in_dyn_function_where_parent_has_active_coroutine() {
    let push_from_global = common::gen_push_global();
    let set_dyn_call = common::gen_set_dyn_call();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Yield(Slot::Local(0)),
            Op::Finish,
        ],
    };

    let child = Fun {
        name: "child".into(),
        instrs: vec![
            Op::Call(2, vec![]),
            Op::Resume(0),
            Op::FinishSetBranch(0),
            Op::Branch(5),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(2, vec![]),
            Op::Gen(0, vec![0]),
            Op::Gen(1, vec![0]),
            Op::DynCall(vec![]),
            Op::Call(1, vec![]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new(
        vec![main, child, co], 
        vec![push_from_global, set_dyn_call]);

    vm.with_globals(vec![1, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 5);
}

#[test]
fn should_branch_on_finished_coroutine_in_resumed_coroutine_where_parent_has_active_coroutine() {
    let push_from_global = common::gen_push_global();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Yield(Slot::Local(0)),
            Op::Finish,
        ],
    };

    let child = Fun {
        name: "child".into(),
        instrs: vec![
            Op::Call(2, vec![]),
            Op::Gen(0, vec![0]),
            Op::Yield(Slot::Local(0)),
            Op::Resume(0),
            Op::FinishSetBranch(0),
            Op::Branch(7),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Yield(Slot::Local(1)),
            Op::Finish,
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(2, vec![]),
            Op::Call(1, vec![]),
            Op::Resume(1),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new(
        vec![main, child, co], 
        vec![push_from_global]);

    vm.with_globals(vec![1, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 5);
}
