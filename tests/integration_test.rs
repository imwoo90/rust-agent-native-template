#![allow(missing_docs)]

use rust_agent_native_template::Calculator;
use rust_agent_native_template::example::calculator::CalculatorError;

#[test]
fn test_calculator_chained_operations() {
    let mut calc = Calculator::new();
    calc.add(50.0).subtract(10.0).multiply(2.0);
    assert_eq!(calc.value(), 80.0);

    let res = calc.divide(4.0);
    assert!(res.is_ok());
    assert_eq!(calc.value(), 20.0);
}

#[test]
fn test_calculator_division_by_zero_error() {
    let mut calc = Calculator::new();
    let res = calc.divide(0.0);
    assert_eq!(res, Err(CalculatorError::DivisionByZero));
}
