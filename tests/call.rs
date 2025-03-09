
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;

#[test]
fn should_dynamic_call() {
    const ONE : usize = 0;
    const TWO : usize = 1;
    const ADD : usize = 2;
    const PUSH_FROM_GLOBAL : usize = 3;

    let add = common::gen_add();
    let push_from_global = common::gen_push_global();

    let set_dyn_call_two = GenOp { 
        name: "set two".into(),
        op: | env, _ | {
            *env.dyn_call = Some(2);
            Ok(())
        },
    };

    let set_dyn_call_one = GenOp { 
        name: "set one".into(),
        op: | env, _ | {
            *env.dyn_call = Some(1);
            Ok(())
        },
    };

    let two = Fun { 
        name: "two".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(1)]), // get 2
            Op::Gen(ADD, vec![Slot::Local(0), Slot::Local(1)]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let one = Fun { 
        name: "one".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(0)]), // get 1
            Op::Gen(ADD, vec![Slot::Local(0), Slot::Local(1)]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(2)]), // get 7 (now local 0)
            Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(3)]), // get 17 (now local 1)
            Op::Gen(ONE, vec![]),
            Op::DynCall(vec![Slot::Local(0)]), // should add 1 to 7 
            Op::PushRet,                       // local 2 is now 8
            Op::Gen(TWO, vec![]),
            Op::DynCall(vec![Slot::Local(1)]), // should add 2 to 17 
            Op::PushRet,                       // local 3 is now 19
            Op::Gen(ADD, vec![Slot::Local(2), Slot::Local(3)]), // should add 19 to 8
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, one, two], 
        vec![set_dyn_call_one, set_dyn_call_two, add, push_from_global]);

    vm.with_globals(vec![1, 2, 7, 17]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 27);
}

#[test]
fn should_handle_multiple_calls() {
    const MUL : usize = 0;
    const PUSH_FROM_GLOBAL : usize = 1;
    const BZ : usize = 2;
    const DEC : usize = 3;

    let mul = common::gen_mul();
    let push_from_global = common::gen_push_global();
    let bz = common::gen_set_branch_on_zero();
    let dec = common::gen_dec();

    let factorial = Fun { 
        name: "fact".into(),
        instrs: vec![
            Op::Gen(DEC, vec![Slot::Local(0)]),
            Op::PushRet,
            Op::Gen(BZ, vec![Slot::Local(1)]),
            Op::Branch(8),
            Op::Call(1, vec![Slot::Local(1)]),
            Op::PushRet,
            Op::Gen(MUL, vec![Slot::Local(0), Slot::Local(2)]),
            Op::ReturnSlot(Slot::Return),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![Slot::Local(0)]),
            Op::Call(1, vec![Slot::Local(0)]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, factorial], 
        vec![mul, push_from_global, bz, dec]);

    vm.with_globals(vec![5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 120);
}

#[test]
fn should_return() {
    const INTO_G : usize = 0;
    const FROM_G : usize = 1;
    const ADD : usize = 2;

    let push_from_global = common::gen_push_global();
    let push_into_global = common::gen_push_into_global();
    let add = common::gen_add();

    let other = Fun { 
        name: "other".into(),
        instrs: vec![
            Op::Gen(ADD, vec![Slot::Local(0), Slot::Local(1)]),
            Op::PushRet,
            Op::Gen(INTO_G, vec![Slot::Local(2)]),
            Op::Return,
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(FROM_G, vec![Slot::Local(1)]),
            Op::Gen(FROM_G, vec![Slot::Local(2)]),
            Op::Call(1, vec![Slot::Local(0), Slot::Local(1)]),
            Op::Gen(FROM_G, vec![Slot::Local(3)]), // from global slot 3
            Op::ReturnSlot(Slot::Local(2)), // from local slot 2
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, other], 
        vec![push_into_global, push_from_global, add]);

    vm.with_globals(vec![0, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 8);
}

#[test]
fn should_order_params() {
    let push = common::gen_push_global();
    let bz = common::gen_set_branch_on_zero();

    let other = Fun { 
        name: "other".into(),
        instrs: vec![
            Op::Gen(1, vec![Slot::Local(2)]),
            Op::Branch(3),
            Op::ReturnSlot(Slot::Local(0)),
            Op::ReturnSlot(Slot::Local(1)),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![Slot::Local(0)]),
            Op::Gen(0, vec![Slot::Local(1)]),
            Op::Gen(0, vec![Slot::Local(2)]),
            Op::Call(1, vec![Slot::Local(2), Slot::Local(1), Slot::Local(0)]), // other(5, 3, 0)
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, other], 
        vec![push, bz]);

    vm.with_globals(vec![0, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_call_with_params() {
    let push = common::gen_push_global();
    let add = common::gen_add();

    let add_up = Fun { 
        name: "add_up".into(),
        instrs: vec![
            Op::Gen(1, vec![Slot::Local(0), Slot::Local(1)]),
            Op::PushRet,
            Op::Gen(1, vec![Slot::Local(2), Slot::Local(3)]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![Slot::Local(0)]),
            Op::Gen(0, vec![Slot::Local(1)]),
            Op::Gen(0, vec![Slot::Local(2)]),
            Op::Call(1, vec![Slot::Local(0), Slot::Local(1), Slot::Local(2)]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, add_up], 
        vec![push, add]);

    vm.with_globals(vec![2, 3, 5]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 10);
}

#[test]
fn should_call_and_return() {

    let push : GenOp<u8, u8> = GenOp {
        name : "push".into(),
        op: |env, _ | { 
            env.locals.last_mut().unwrap().push(9);
            Ok(())
        },
    };

    let ret_nine = Fun {
        name : "ret_nine".into(),
        instrs: vec![
            Op::Gen(0, vec![]),
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(vec![main, ret_nine], vec![push]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 9);
}