
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

    let set_dyn_call_two = GenOp::Frame { 
        name: "set two".into(),
        op: | frame, _ | {
            frame.dyn_call = Some(2);
            Ok(None)
        },
    };

    let set_dyn_call_one = GenOp::Frame { 
        name: "set one".into(),
        op: | frame, _ | {
            frame.dyn_call = Some(1);
            Ok(None)
        },
    };

    let two = Fun { 
        name: "two".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![1]), // get 2
            Op::Gen(ADD, vec![0, 1]),
            Op::PushRet,
            Op::ReturnLocal(2),
        ],
    };

    let one = Fun { 
        name: "one".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![0]), // get 1
            Op::Gen(ADD, vec![0, 1]),
            Op::PushRet,
            Op::ReturnLocal(2),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![2]), // get 7 (now local 0)
            Op::Gen(PUSH_FROM_GLOBAL, vec![3]), // get 17 (now local 1)
            Op::Gen(ONE, vec![]),
            Op::DynCall(vec![0]),              // should add 1 to 7 
            Op::PushRet,                       // local 2 is now 8
            Op::Gen(TWO, vec![]),
            Op::DynCall(vec![1]),              // should add 2 to 17 
            Op::PushRet,                       // local 3 is now 19
            Op::Gen(ADD, vec![2, 3]),          // should add 19 to 8
            Op::PushRet,
            Op::ReturnLocal(4),
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
            Op::Gen(DEC, vec![0]),
            Op::PushRet,
            Op::Gen(BZ, vec![1]),
            Op::Branch(9),
            Op::Call(1, vec![1]),
            Op::PushRet,
            Op::Gen(MUL, vec![0, 2]),
            Op::PushRet,
            Op::ReturnLocal(3),
            Op::ReturnLocal(0),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(PUSH_FROM_GLOBAL, vec![0]),
            Op::Call(1, vec![0]),
            Op::PushRet,
            Op::ReturnLocal(1),
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
            Op::Gen(ADD, vec![0, 1]),
            Op::PushRet,
            Op::Gen(INTO_G, vec![2]),
            Op::Return,
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(FROM_G, vec![1]),
            Op::Gen(FROM_G, vec![2]),
            Op::Call(1, vec![0, 1]),
            Op::Gen(FROM_G, vec![3]),  // from global slot 3
            Op::ReturnLocal(2),        // from local slot 2
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
            Op::Gen(1, vec![2]),
            Op::Branch(3),
            Op::ReturnLocal(0),
            Op::ReturnLocal(1),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Call(1, vec![2, 1, 0]), // other(5, 3, 0)
            Op::PushRet,
            Op::ReturnLocal(3),
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
fn should_order_params_with_dyn_call() {
    let push = common::gen_push_global();
    let add = common::gen_add();
    let mul = common::gen_mul();
    let set_dyn_call = common::gen_set_dyn_call();

    let other = Fun { 
        name: "other".into(),
        instrs: vec![
            Op::Gen(2, vec![0, 1]), // 3 + 5
            Op::PushRet,
            Op::Gen(3, vec![3, 2]), // 8 * 7
            Op::PushRet,
            Op::ReturnLocal(4),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Gen(0, vec![3]),
            Op::Gen(0, vec![0]),
            Op::Gen(1, vec![3]),
            Op::DynCall(vec![0, 1, 2]), // other(3, 5, 7)
            Op::PushRet,
            Op::ReturnLocal(4),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new(
        vec![main, other], 
        vec![push, set_dyn_call, add, mul]);

    vm.with_globals(vec![1, 3, 5, 7]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 56);
}

#[test]
fn should_call_with_params() {
    let push = common::gen_push_global();
    let add = common::gen_add();

    let add_up = Fun { 
        name: "add_up".into(),
        instrs: vec![
            Op::Gen(1, vec![0, 1]),
            Op::PushRet,
            Op::Gen(1, vec![2, 3]),
            Op::PushRet,
            Op::ReturnLocal(4),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Call(1, vec![0, 1, 2]),
            Op::PushRet,
            Op::ReturnLocal(3),
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

    let push : GenOp<u8, u8> = GenOp::Local {
        name : "push".into(),
        op: |locals, _ | { 
            locals.push(9);
            Ok(None)
        },
    };

    let ret_nine = Fun {
        name : "ret_nine".into(),
        instrs: vec![
            Op::Gen(0, vec![]),
            Op::ReturnLocal(0),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::PushRet,
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(vec![main, ret_nine], vec![push]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 9);
}