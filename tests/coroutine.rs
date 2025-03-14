
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

// TODO 
// should handle params
// call
// dyn call
// multiple coroutines interleaved inside of a coroutine
// finish set branch kills off finished coroutine
// finish set branch doesnt kill off active coroutine
// resuming coroutine pulls coroutine out of order
// yielding or finishing coroutine puts it in the end of the coroutine list