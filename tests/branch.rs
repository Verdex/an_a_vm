
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