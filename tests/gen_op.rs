
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;

#[test]
fn should_return_from_local() {
    let op = GenOp::Local {
        name: "op".into(),
        op: | locals, params |  { 
            Ok(Some(3))
        },
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![]),
            Op::PushRet,
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main],
        vec![op]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}