
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
            Op::Yield(0),
            Op::Finish,
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
            Op::Yield(0),
            Op::Gen(0, vec![0]),
            Op::Gen(1, vec![1, 2]),
            Op::PushRet,
            Op::Yield(3),
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
            Op::Yield(3),
            Op::Gen(2, vec![3, 2]),
            Op::PushRet,
            Op::Yield(4),
            Op::Finish,
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
            Op::Resume(0),
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
            Op::Yield(3),
            Op::Gen(2, vec![3, 2]),
            Op::PushRet,
            Op::Yield(4),
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
            Op::DynCall(vec![1, 2, 3]),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::Resume(0),
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

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Yield(0),
            Op::Yield(0),
            Op::Finish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::FinishSetBranch(0),
            Op::Branch(6),
            Op::Resume(0),
            Op::Gen(0, vec![1]),
            Op::ReturnLocal(0),
            Op::Gen(0, vec![0]),
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main, co],
        vec![push_from_global]);

    vm.with_globals(vec![1, 2]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 2);
}

#[test]
fn should_remove_finished_coroutine_for_finish_set_branch() {
    let push_from_global = common::gen_push_global();
    let add = common::gen_add();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Yield(0),
            Op::Yield(0),
            Op::Yield(1),
            Op::Finish,
        ],
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Call(1, vec![]),
            Op::Call(1, vec![]),
            Op::Call(1, vec![]),
            Op::Resume(0),
            Op::Resume(2),
            Op::Resume(2),
            Op::Resume(0),
            Op::Resume(2),
            Op::Resume(2),
            Op::FinishSetBranch(2),
            Op::FinishSetBranch(1),
            Op::Resume(0),
            Op::PushRet, // 1 on stack
            Op::Resume(0),
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
        vec![push_from_global, add]);

    vm.with_globals(vec![1, 2]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_move_coroutine_position_on_resume_yield() {
    const END : usize = 40;

    let push_from_global = common::gen_push_global();
    let set_branch = common::gen_set_branch();
    let unset_branch_on_equal = common::gen_unset_branch_on_equal();

    let co = Fun {
        name: "co".into(),
        instrs: vec![
            Op::Yield(0),
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

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![4, 0]),
            Op::Branch(END),
            Op::Drop(4),

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![4, 1]),
            Op::Branch(END),
            Op::Drop(4),

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![4, 2]),
            Op::Branch(END),
            Op::Drop(4),

            Op::Resume(0),
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

#[test]
fn should_handle_coroutine_with_interleaved_coroutines() {
    let push_from_global = common::gen_push_global();
    let set_branch = common::gen_set_branch();
    let add = common::gen_add();

    let blarg = GenOp {
        name: "balfg".into(),
        op: | env, params |  { 
            if let [s] = &params[..] {
                let a = &env.locals.last().unwrap()[*s];
                println!("{:?}", a);
            }
            Ok(())
        },
    };

    let inf = Fun {
        name: "inf".into(),
        instrs: vec![
            Op::Yield(0),
            Op::Gen(1, vec![]),
            Op::Branch(0),
        ],
    };

    let com = Fun {
        name: "com".into(),
        instrs: vec![
            Op::Gen(0, vec![0]),
            Op::Gen(0, vec![1]),
            Op::Gen(0, vec![2]),
            Op::Call(2, vec![0]),
            Op::PushRet,
            Op::Yield(3),
            Op::Call(2, vec![1]),
            Op::PushRet,
            Op::Gen(2, vec![3, 4]),
            Op::PushRet,
            Op::Yield(5),
            Op::Call(2, vec![2]),
            Op::PushRet,
            Op::Gen(2, vec![5, 6]),
            Op::PushRet,
            Op::Yield(7),
            Op::Drop(6),
            Op::Drop(5),
            Op::Drop(4),
            Op::Drop(3),
            Op::Drop(2),
            Op::Drop(1),
            Op::Drop(0),
            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet,
            Op::Yield(0),
            Op::Gen(1, vec![]),
            Op::Branch(23),
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
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 1 + 3

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 4 + 6

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0), 
            Op::PushRet, // 10 + 9

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 19 + 13

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 32 + 18

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 50 + 21

            Op::Resume(0),
            Op::PushRet,
            Op::Gen(2, vec![0, 1]),
            Op::Drop(0),
            Op::Drop(0),
            Op::PushRet, // 71 + 25

            Op::Resume(0),
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
        vec![push_from_global, set_branch, add, blarg]);

    vm.with_globals(vec![1, 2, 3]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 126);
}
