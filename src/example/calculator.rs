//! # Calculator Engine Module
//!
//! ## Overview
//! Provides fundamental arithmetic operations designed as an executable, compiler-verified example
//! of the Agent-Native pattern. Includes unit tests, executable doctests, and typed error handling.
//!
//! ## Search Tags
//! #calculator, #math, #arithmetic

/// A simple accumulator-based calculator engine.
///
/// Demonstrates the living LLM-Wiki philosophy: doc comments are not merely comments,
/// but compiler-verified executable doctests and type-checked links.
///
/// # Examples
///
/// ```rust
/// use rust_agent_native_template::Calculator;
///
/// let mut calc = Calculator::new();
/// calc.add(10.0);
/// calc.multiply(2.0);
/// assert_eq!(calc.value(), 20.0);
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Calculator {
    current_value: f64,
}

/// Errors that can occur during calculation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalculatorError {
    /// Attempted division by zero.
    DivisionByZero,
}

impl std::fmt::Display for CalculatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivisionByZero => write!(f, "cannot divide by zero"),
        }
    }
}

impl std::error::Error for CalculatorError {}

impl Calculator {
    /// Creates a new [`Calculator`] initialized with an accumulator value of `0.0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_agent_native_template::Calculator;
    ///
    /// let calc = Calculator::new();
    /// assert_eq!(calc.value(), 0.0);
    /// ```
    pub fn new() -> Self {
        Self { current_value: 0.0 }
    }

    /// Returns the current accumulated value.
    pub fn value(&self) -> f64 {
        self.current_value
    }

    /// Adds `rhs` to the accumulator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_agent_native_template::Calculator;
    ///
    /// let mut calc = Calculator::new();
    /// calc.add(5.0);
    /// assert_eq!(calc.value(), 5.0);
    /// ```
    pub fn add(&mut self, rhs: f64) -> &mut Self {
        self.current_value += rhs;
        self
    }

    /// Subtracts `rhs` from the accumulator.
    pub fn subtract(&mut self, rhs: f64) -> &mut Self {
        self.current_value -= rhs;
        self
    }

    /// Multiplies the accumulator by `rhs`.
    pub fn multiply(&mut self, rhs: f64) -> &mut Self {
        self.current_value *= rhs;
        self
    }

    /// Divides the accumulator by `rhs`. Returns [`CalculatorError::DivisionByZero`] if `rhs == 0.0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_agent_native_template::Calculator;
    ///
    /// let mut calc = Calculator::new();
    /// calc.add(10.0);
    /// assert!(calc.divide(2.0).is_ok());
    /// assert_eq!(calc.value(), 5.0);
    /// assert!(calc.divide(0.0).is_err());
    /// ```
    pub fn divide(&mut self, rhs: f64) -> Result<&mut Self, CalculatorError> {
        if rhs == 0.0 {
            return Err(CalculatorError::DivisionByZero);
        }
        self.current_value /= rhs;
        Ok(self)
    }

    /// Resets the accumulator back to `0.0`.
    pub fn reset(&mut self) {
        self.current_value = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let calc = Calculator::new();
        assert_eq!(calc.value(), 0.0);
    }

    #[test]
    fn test_addition_and_subtraction() {
        let mut calc = Calculator::new();
        calc.add(15.0).subtract(5.0);
        assert_eq!(calc.value(), 10.0);
    }

    #[test]
    fn test_multiplication() {
        let mut calc = Calculator::new();
        calc.add(4.0).multiply(2.5);
        assert_eq!(calc.value(), 10.0);
    }

    #[test]
    fn test_division_success_and_error() {
        let mut calc = Calculator::new();
        calc.add(20.0);
        assert!(calc.divide(4.0).is_ok());
        assert_eq!(calc.value(), 5.0);

        let err = calc.divide(0.0);
        assert_eq!(err, Err(CalculatorError::DivisionByZero));
        // Accumulator remains unchanged on error
        assert_eq!(calc.value(), 5.0);
    }

    #[test]
    fn test_reset() {
        let mut calc = Calculator::new();
        calc.add(100.0);
        calc.reset();
        assert_eq!(calc.value(), 0.0);
    }
}
