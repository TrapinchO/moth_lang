use crate::backend::interpreter::Interpreter;
use crate::run;
use crate::backend::value::{get_builtins, ValueType};

fn run_code(code: &str, val: &str) -> Option<ValueType> {
    let mut interp = Interpreter::new(get_builtins());
    run(&mut interp, code, false).ok()?;
    interp.get_val(val.to_string())
}

#[test]
fn blank() {
    let mut interp = Interpreter::new(get_builtins());
    let res = run(&mut interp, "", false);
    assert_eq!(res.is_ok(), true);
}

#[test]
fn unit() {
    assert_eq!(
        run_code("let x = ();", "x"),
        Some(ValueType::Unit)
    );
}
#[test]
fn expr() {
    assert_eq!(
        run_code("let x = 1 + 2 * 2 - 6 / 3;", "x"),
        Some(ValueType::Int(3))
    );
}

#[test]
fn list() {
    assert_eq!(
        run_code("let x = [1, 2, 3]; x[1] = 1.1; let y = x[1];", "y"),
        Some(ValueType::Float(1.1))
    );
}

#[test]
fn t() {
    let mut interp = Interpreter::new(get_builtins());
    run(&mut interp, "let x = 2;", false).unwrap();
    assert_eq!(ValueType::Int(2), interp.get_val("x".to_string()).unwrap());
}

#[test]
fn t2() {
    assert_eq!(run_code("let x = 10 + 5;", "x").unwrap(), ValueType::Int(15));
}

#[test]
fn apply_fun() {
    assert_eq!(
        run_code("
fun fact(n) {
    let total = 1;
    while n > 1 {
        total = total * n;
        n = n - 1;
    }
    return total;
}
fun <<(f, g) {
    fun a(x) {
        return f(g(x));
    }
    return a;
}
let x = (fact << len)([1, 2, 3, 4]);",
            "x"
        ),
        Some(ValueType::Int(24))
    );
}

#[test]
fn closure() {
    assert_eq!(
        run_code("
fun n() {
    let x = 0;
    fun g() {
        x = x + 1;
        return x;
    }
    return g;
}
let f = n();
f();
f();
f();
f();
let x = f();", "x"),
        Some(ValueType::Int(5))
    )
}

#[test]
fn unary_operators() {
    assert_eq!(
        run_code("let x = !(1 * 10 == -10);", "x"),
        Some(ValueType::Bool(true))
    )
}

#[test]
fn structs() {
    assert_eq!(
        run_code("
struct Point {
    x,
    y,
}
let p = Point(1, -20);
p.x = 10 * 2 + 20;
let x = p.x + p.y;
            ", "x"),
        Some(ValueType::Int(20))
    )
}

#[test]
fn unknown_field() {
    assert_eq!(
        run_code("
struct Point {
    x,
    y,
}
let p = Point(1, -20);
p.z;
            ", "x"),
        None
    )
}


