
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
            if let Slot::Local(0) = ps[0] {
                env.locals.last_mut().unwrap().push(0);
            }
            if let Slot::Local(1) = ps[0] {
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
            Op::Gen(P, vec![Slot::Local(0)]),
            Op::ReturnSlot(Slot::Local(0)),

            Op::Gen(U, vec![]),
            Op::Branch(8),         
            Op::Gen(P, vec![Slot::Local(1)]),
            Op::ReturnSlot(Slot::Local(0)),

            Op::Gen(P, vec![Slot::Local(0)]),
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
            Op::Gen(PUSH, vec![Slot::Local(0)]),
            Op::Gen(PUSH, vec![Slot::Local(0)]),
            Op::Gen(PUSH, vec![Slot::Local(1)]),
            Op::Gen(SBE, vec![Slot::Local(0), Slot::Local(2)]),
            Op::Branch(19), 
            Op::Gen(INC, vec![Slot::Local(1)]),
            Op::PushRet,
            Op::Swap(1, 3),
            Op::Drop(3),
            Op::Gen(INC, vec![Slot::Local(1)]),
            Op::PushRet,
            Op::Swap(1, 3),
            Op::Drop(3),
            Op::Gen(DEC, vec![Slot::Local(2)]),
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
            Op::Gen(0, vec![Slot::Local(0)]),
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
            Op::Gen(0, vec![Slot::Local(1)]),
            Op::Gen(0, vec![Slot::Local(2)]),
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
            Op::Gen(0, vec![Slot::Local(0)]),
            Op::Gen(0, vec![Slot::Local(1)]),
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
            Op::Gen(0, vec![Slot::Local(0)]),
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
            Op::Gen(0, vec![Slot::Local(1)]),
            Op::Gen(0, vec![Slot::Local(2)]),
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

// TODO branch on finnished coroutine inside of function while return function has active coroutine in same slot
// TODO same as above but for already resumed coroutine 
// TODO same as above but for a dyn call