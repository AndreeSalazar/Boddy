#![no_std]
//! Ejercicios y expresiones Rust cerradas. No interpreta ni ejecuta texto Rust.
pub const TRAIN: &str = include_str!("../../../training/corpora/boddy-rust-v1.txt");
pub const VALIDATION: &str = include_str!("../../../training/evals/boddy-rust-v1.txt");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expression {
    Add,
    Subtract,
    Multiply,
    Maximum,
    Minimum,
    AbsA,
    AbsB,
    Left,
    Right,
}
pub const EXPRESSIONS: [Expression; 9] = [
    Expression::Add,
    Expression::Subtract,
    Expression::Multiply,
    Expression::Maximum,
    Expression::Minimum,
    Expression::AbsA,
    Expression::AbsB,
    Expression::Left,
    Expression::Right,
];
impl Expression {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Subtract => "sub",
            Self::Multiply => "mul",
            Self::Maximum => "max",
            Self::Minimum => "min",
            Self::AbsA => "abs_a",
            Self::AbsB => "abs_b",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
    pub const fn source(self) -> &'static str {
        match self {
            Self::Add => "a + b",
            Self::Subtract => "a - b",
            Self::Multiply => "a * b",
            Self::Maximum => "a.max(b)",
            Self::Minimum => "a.min(b)",
            Self::AbsA => "a.abs()",
            Self::AbsB => "b.abs()",
            Self::Left => "a",
            Self::Right => "b",
        }
    }
    pub fn evaluate(self, a: i32, b: i32) -> Option<i32> {
        match self {
            Self::Add => a.checked_add(b),
            Self::Subtract => a.checked_sub(b),
            Self::Multiply => a.checked_mul(b),
            Self::Maximum => Some(a.max(b)),
            Self::Minimum => Some(a.min(b)),
            Self::AbsA => a.checked_abs(),
            Self::AbsB => b.checked_abs(),
            Self::Left => Some(a),
            Self::Right => Some(b),
        }
    }
}
pub struct Case {
    pub a: i32,
    pub b: i32,
    pub expected: i32,
}
pub struct Exercise {
    pub id: &'static str,
    pub goal: &'static str,
    pub tag: &'static str,
    pub cases: [Case; 5],
}
pub struct Checked {
    pub passed: usize,
    pub total: usize,
    pub first_failure: Option<usize>,
}
pub fn verify(exercise: &Exercise, expression: Expression) -> Checked {
    let mut passed = 0;
    let mut first_failure = None;
    for (i, c) in exercise.cases.iter().enumerate() {
        if expression.evaluate(c.a, c.b) == Some(c.expected) {
            passed += 1;
        } else if first_failure.is_none() {
            first_failure = Some(i);
        }
    }
    Checked {
        passed,
        total: exercise.cases.len(),
        first_failure,
    }
}
const fn c(a: i32, b: i32, expected: i32) -> Case {
    Case { a, b, expected }
}
pub static EXERCISES: [Exercise; 6] = [
    Exercise {
        id: "suma",
        goal: "Devolver la suma de a y b",
        tag: "suma",
        cases: [
            c(2, 3, 5),
            c(-4, 9, 5),
            c(0, 0, 0),
            c(8, -2, 6),
            c(-3, -7, -10),
        ],
    },
    Exercise {
        id: "resta",
        goal: "Restar b de a",
        tag: "resta",
        cases: [
            c(2, 3, -1),
            c(-4, 9, -13),
            c(0, 0, 0),
            c(8, -2, 10),
            c(-3, -7, 4),
        ],
    },
    Exercise {
        id: "producto",
        goal: "Multiplicar a por b",
        tag: "producto",
        cases: [
            c(2, 3, 6),
            c(-4, 9, -36),
            c(0, 5, 0),
            c(8, -2, -16),
            c(-3, -7, 21),
        ],
    },
    Exercise {
        id: "mayor",
        goal: "Devolver el mayor de a y b",
        tag: "mayor",
        cases: [
            c(2, 3, 3),
            c(-4, 9, 9),
            c(0, 0, 0),
            c(8, -2, 8),
            c(-3, -7, -3),
        ],
    },
    Exercise {
        id: "menor",
        goal: "Devolver el menor de a y b",
        tag: "menor",
        cases: [
            c(2, 3, 2),
            c(-4, 9, -4),
            c(0, 0, 0),
            c(8, -2, -2),
            c(-3, -7, -7),
        ],
    },
    Exercise {
        id: "absoluto",
        goal: "Devolver el valor absoluto de a",
        tag: "absoluto",
        cases: [
            c(2, 3, 2),
            c(-4, 9, 4),
            c(0, 0, 0),
            c(8, -2, 8),
            c(-3, -7, 3),
        ],
    },
];
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_cases_distinguish_all_solutions() {
        let expected = [
            Expression::Add,
            Expression::Subtract,
            Expression::Multiply,
            Expression::Maximum,
            Expression::Minimum,
            Expression::AbsA,
        ];
        for (task, answer) in EXERCISES.iter().zip(expected) {
            let passing: usize = EXPRESSIONS
                .iter()
                .filter(|op| verify(task, **op).passed == 5)
                .count();
            assert_eq!(passing, 1);
            assert_eq!(verify(task, answer).passed, 5);
        }
    }
    #[test]
    fn overflow_is_rejected_instead_of_wrapped() {
        assert_eq!(Expression::Add.evaluate(i32::MAX, 1), None);
        assert_eq!(Expression::Subtract.evaluate(i32::MIN, 1), None);
        assert_eq!(Expression::Multiply.evaluate(i32::MAX, 2), None);
        assert_eq!(Expression::AbsA.evaluate(i32::MIN, 0), None);
    }
}
