
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
            Op::CoYield(0),
            Op::CoFinish,
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
            Op::CoYield(0),
            Op::Gen(0, vec![0]),
            Op::Gen(1, vec![1, 2]),
            Op::PushRet,
            Op::CoYield(3),
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::PushRet,
            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(1, vec![0, 1]),
            Op::PushRet,
            Op::ReturnLocal(2),
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
            Op::CoYield(3),
            Op::Gen(2, vec![3, 2]),
            Op::PushRet,
            Op::CoYield(4),
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Call(1, vec![0, 1, 2]),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![1, 0]),
            Op::PushRet,
            Op::ReturnLocal(2), 
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, add, mul]);

    vm.with_globals(vec![3, 5, 7]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 448);
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
            Op::CoYield(3),
            Op::Gen(2, vec![3, 2]),
            Op::PushRet,
            Op::CoYield(4),
            Op::CoFinish,
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
            Op::DynCall(vec![1, 2, 3]),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::PushRet,
            Op::ReturnLocal(2), 
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, add, mul, set_dyn]);

    vm.with_globals(vec![1, 3, 5, 7]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 448);
}

#[test]
fn should_preserve_active_coroutine_for_finish_set_branch() {
    let push_from_global = common::gen_push_global();
    let set_branch_on_finish = common::gen_set_branch_on_finish();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::CoYield(0),
            Op::CoYield(0),
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::Gen(1, vec![0]),
            Op::Branch(6),
            Op::CoResume(0),
            Op::Gen(0, vec![1]),
            Op::ReturnLocal(0),
            Op::Gen(0, vec![0]),
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, set_branch_on_finish]);

    vm.with_globals(vec![1, 2]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 2);
}

#[test]
fn should_remove_finished_coroutine_for_finish_set_branch() {
    let push_from_global = common::gen_push_global();
    let add = common::gen_add();
    let set_branch_on_finish = common::gen_set_branch_on_finish();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::CoYield(0),
            Op::CoYield(0),
            Op::CoYield(1),
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::Call(1, vec![]),
            Op::Call(1, vec![]),
            Op::CoResume(0),
            Op::CoResume(0),
            Op::CoResume(0),
            Op::CoResume(1),
            Op::CoResume(1),
            Op::CoResume(1),
            Op::Gen(2, vec![0]),
            Op::CoDrop(0),
            Op::Gen(2, vec![0]),
            Op::CoDrop(0),
            Op::CoResume(0),
            Op::PushRet, // 1 on stack
            Op::CoResume(0),
            Op::PushRet,  // 2 on stack
            Op::Gen(1, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, add, set_branch_on_finish]);

    vm.with_globals(vec![1, 2]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_not_move_coroutine_position_on_resume_yield() {
    const END : usize = 40;

    let push_from_global = common::gen_push_global();
    let set_branch = common::gen_set_branch();
    let unset_branch_on_equal = common::gen_unset_branch_on_equal();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::CoYield(0),
            Op::Gen(1, vec![]),
            Op::Branch(0),
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Gen(0, vec![3]),

            Op::Call(1, vec![0]),
            Op::PushRet,
            Op::Gen(2, vec![4, 0]),
            Op::Branch(END),
            Op::Drop(4),

            Op::Call(1, vec![1]),
            Op::PushRet,
            Op::Gen(2, vec![4, 1]),
            Op::Branch(END), 
            Op::Drop(4),

            Op::Call(1, vec![2]),
            Op::PushRet,
            Op::Gen(2, vec![4, 2]),
            Op::Branch(END), 
            Op::Drop(4),

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![4, 0]),
            Op::Branch(END),
            Op::Drop(4),

            Op::CoResume(1),
            Op::PushRet,
            Op::Gen(2, vec![4, 1]),
            Op::Branch(END),
            Op::Drop(4),

            Op::CoResume(2),
            Op::PushRet,
            Op::Gen(2, vec![4, 2]),
            Op::Branch(END),
            Op::Drop(4),

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![4, 0]),
            Op::Branch(END),
            Op::Drop(4),

            Op::ReturnLocal(0),
            Op::ReturnLocal(3),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global, set_branch, unset_branch_on_equal]);

    vm.with_globals(vec![1, 2, 3, 9]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 1);
}

// TODO add test of coroutine that has coroutines and the inner coroutines produce different values the 
// more times they're iterated through

// TODO a recursive coroutine ought to work correctly

#[test]
fn should_handle_coroutine_with_interleaved_coroutines() {
    let push_from_global = common::gen_push_global();
    let set_branch = common::gen_set_branch();
    let add = common::gen_add();

    let inf = Fun {
        name: "inf".into(),
        instrs: vec![
            Op::CoYield(0),
            Op::Gen(1, vec![]),
            Op::Branch(0),
        ],
    };

    let com = Fun {
        name: "com".into(),
        instrs: vec![
            Op::Gen(0, vec![0]), // 1
            Op::Gen(0, vec![1]), // 2
            Op::Gen(0, vec![2]), // 3
            Op::Call(2, vec![0]), // Coroutine 0 returns 1 forever
            Op::PushRet,
            Op::CoYield(3), // yield 1
            Op::Call(2, vec![1]), // coroutine 1 returns 2 forever
            Op::PushRet,
            Op::Gen(2, vec![3, 4]), // 1 + 2
            Op::PushRet,
            Op::CoYield(5), // yield 3
            Op::Call(2, vec![2]), // coroutine 2 returns 3 forever
            Op::PushRet,
            Op::Gen(2, vec![5, 6]), // 3 + 3
            Op::PushRet,
            Op::CoYield(7), // yield 6
            Op::Drop(6),
            Op::Drop(5),
            Op::Drop(4),
            Op::Drop(3),
            Op::Drop(2),
            Op::Drop(1),
            Op::Drop(0),
            Op::CoResume(0), // line 23
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::CoResume(1),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::CoYield(0),
            Op::Gen(1, vec![]),
            Op::CoSwap(0, 2),
            Op::CoSwap(1, 2),
            Op::Branch(23),
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::PushRet,
            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 1 + 3

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]), 
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 4 + 6

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0), 
            Op::PushRet, // 10 + 9

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 19 + 13

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 32 + 18

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 50 + 21

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 71 + 25

            Op::CoResume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 96 + 20

            Op::ReturnLocal(0), // 126
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, com, inf],
        vec![push_from_global, set_branch, add]);

    vm.with_globals(vec![1, 2, 3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 126);
}

#[test]
fn should_handle_coroutine_with_immediate_finish() {
    let set_branch_on_finish = common::gen_set_branch_on_finish();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::Gen(0, vec![0]),
            Op::Branch(4),
            Op::PushLocal(1),
            Op::PushLocal(3),
            Op::ReturnLocal(0),
        ]
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![set_branch_on_finish]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_drop_coroutine() {

    let co_count = GenOp::Frame {
        name: "co_count".into(),
        op: |frame, _| {
            Ok(Some(frame.coroutines.len()))
        },
    };

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::CoDrop(0),
            Op::Gen(0, vec![]),
            Op::PushRet,
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![co_count]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 0);
}

#[test]
fn should_swap_coroutine() {
    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::PushLocal(1),
            Op::PushLocal(2),
            Op::PushLocal(3),
            Op::CoYield(0),
            Op::CoYield(1),
            Op::CoYield(2),
            Op::CoFinish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]), // yields 1
            Op::Call(1, vec![]), // yields 1
            Op::CoResume(0), // yields 2
            Op::CoSwap(0, 1),
            Op::CoResume(0), // Should yield 2
            Op::PushRet,
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 2);
}

// TODO Add tests for coswap, codup 