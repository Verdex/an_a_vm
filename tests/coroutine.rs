
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;

#[test]
fn should_yield() {
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
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global]);

    vm.with_globals(vec![3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_resume() {
    let push_from_global = common::gen_push_global();
    let add = common::gen_add();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(1, vec![0, 0]),
            Op::PushRet,
            Op::Yield(Slot::Local(0)),
            Op::Gen(0, vec![0]),
            Op::Gen(1, vec![1, 2]),
            Op::Yield(Slot::Return),
            Op::Finish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::PushRet,
            Op::Resume(0),
            Op::PushRet,
            Op::Gen(1, vec![0, 1]),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, add]);

    vm.with_globals(vec![3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 12);
}

#[test]
fn should_handle_params() {
    let push_from_global = common::gen_push_global();
    let add = common::gen_add();
    let mul = common::gen_mul();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(1, vec![0, 1]),
            Op::PushRet,
            Op::Yield(Slot::Local(3)),
            Op::Gen(2, vec![3, 2]),
            Op::Yield(Slot::Return),
            Op::Finish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Call(1, vec![Slot::Local(0), Slot::Local(1), Slot::Local(2)]),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::Resume(0),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, add, mul]);

    vm.with_globals(vec![3, 5, 7]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 56);
}

#[test]
fn should_handle_dyn_call_params() {
    let push_from_global = common::gen_push_global();
    let add = common::gen_add();
    let mul = common::gen_mul();
    let set_dyn = common::gen_set_dyn_call();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(1, vec![0, 1]),
            Op::PushRet,
            Op::Yield(Slot::Local(3)),
            Op::Gen(2, vec![3, 2]),
            Op::Yield(Slot::Return),
            Op::Finish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Gen(0, vec![3]),
            Op::Gen(3, vec![0]),
            Op::DynCall(vec![Slot::Local(1), Slot::Local(2), Slot::Local(3)]),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::Resume(0),
            Op::ReturnSlot(Slot::Return),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, add, mul, set_dyn]);

    vm.with_globals(vec![1, 3, 5, 7]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 56);
}

// TODO 
// multiple coroutines interleaved inside of a coroutine
// finish set branch kills off finished coroutine
// finish set branch doesnt kill off active coroutine
// resuming coroutine pulls coroutine out of order
// yielding or finishing coroutine puts it in the end of the coroutine list