
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;

#[test]
fn should_swap() {
    let push_from_global = common::gen_push_global();
    let mul = common::gen_mul();

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Gen(0, vec![3]),
            Op::Swap(0, 3),
            Op::Swap(1, 2),
            Op::Gen(1, vec![3, 2]),
            Op::PushRet,
            Op::ReturnLocal(4),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new( 
        vec![main],
        vec![push_from_global, mul]);

    vm.with_globals(vec![3, 5, 7, 11]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 15);
}

#[test]
fn should_dup() {
    let push_from_global = common::gen_push_global();

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Dup(0),                      
            Op::ReturnLocal(1),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new( 
        vec![main],
        vec![push_from_global]);

    vm.with_globals(vec![3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_drop() {
    let push_from_global = common::gen_push_global();

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]), // push 3
            Op::Drop(0),         // clear 3 
            Op::Gen(0, vec![1]), // push 7
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new( 
        vec![main],
        vec![push_from_global]);

    vm.with_globals(vec![3, 7]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 7);
}

#[test]
fn should_push_return() {
    let push_from_global = common::gen_push_global();

    let one = Fun { 
        name: "one".into(),
        instrs: vec![
            Op::Gen(0, vec![0]), 
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

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main, one], 
        vec![push_from_global]);

    vm.with_globals(vec![3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_push_value() {
    let main = Fun { 
        name: "main".into(),
        instrs: vec![
            Op::PushValue(3),
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<u8, u8> = Vm::new(
        vec![main], 
        vec![]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}