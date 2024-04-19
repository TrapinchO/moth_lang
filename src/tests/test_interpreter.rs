use crate::interpreter::Interpreter;
use crate::value::{ValueType, get_builtins};
use crate::run;

fn run_code(code: String, val: String) -> Option<ValueType> {
    let mut interp = Interpreter::new(get_builtins());
    run(&mut interp, code, false).unwrap();
    interp.get_val(val)
}

#[test]
fn t() {
    let mut interp = Interpreter::new(get_builtins());
    run(&mut interp, "let x = 2;".to_string(), false).unwrap();
    assert_eq!(ValueType::Int(2), interp.get_val("x".to_string()).unwrap());
}

#[test]
fn t2() {
    assert_eq!(
        run_code("let x = 10 + 5;".to_string(), "x".to_string()).unwrap(),
        ValueType::Int(15)
        );
}
