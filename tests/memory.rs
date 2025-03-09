
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;


#[test]
fn should_push_return() {
    let push_from_global = common::gen_push_global();

    let one = Fun { 
        name: "one".into(),
        instrs: vec![
            Op::Gen(0, vec![Slot::Local(0)]), 
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::PushRet,
            Op::ReturnSlot(Slot::Local(0)),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, one], 
        vec![push_from_global]);

    vm.with_globals(vec![3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}