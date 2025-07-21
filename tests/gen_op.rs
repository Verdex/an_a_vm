
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;
use an_a_vm::error::*;

#[test]
fn should_modify_local() {
    let op = GenOp::Local {
        name: "op".into(),
        op: | locals, params |  { 
            locals[params[0]] += 1;
            Ok(None)
        },
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::PushLocal(0),
            Op::PushLocal(3),
            Op::Gen(0, vec![1]),
            Op::ReturnLocal(1),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main],
        vec![op]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 4);
}

#[test]
fn should_push_local() {
    let op = GenOp::Local {
        name: "op".into(),
        op: | locals, _params |  { 
            locals.push(3);
            Ok(None)
        },
    };

    let main = Fun {
        name: "main".into(),
        instrs: vec![
            Op::Gen(0, vec![]),
            Op::ReturnLocal(0),
        ],
    };

    let mut vm : Vm<usize, usize> = Vm::new( 
        vec![main],
        vec![op]);

    let data = vm.run(0).unwrap().unwrap();

    assert_eq!(data, 3);
}

#[test]
fn should_not_return_from_local() {
    let op = GenOp::Local {
        name: "op".into(),
        op: | _locals, _params |  { 
            Ok(None)
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

    let error = vm.run(0);

    assert!(matches!(error, Err(VmError::AccessMissingReturn(_))));
}

#[test]
fn should_return_from_local() {
    let op = GenOp::Local {
        name: "op".into(),
        op: | _locals, _params |  { 
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