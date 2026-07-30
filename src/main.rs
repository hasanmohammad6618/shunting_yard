use std::fmt;

#[derive(Debug, PartialEq)]
enum Tokens {
    Number(String),
    Operator(char),
    UnaryOperator(char),
    LeftParen(char),
    RightParen,
}

#[derive(Debug)]
enum ParseError {
    UnknownCharacter(char),
    EmptyExpression,
    EmptyParentheses,
    MismatchParentheses,
    DecimalPoint,
    DivisionByZero,
    MissingOperator,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnknownCharacter(chr) => write!(f, "Unknown Character : {}", chr),
            ParseError::EmptyExpression => write!(f, "Expression is Empty"),
            ParseError::EmptyParentheses => write!(f, "Nothing inside parentheses"),
            ParseError::MismatchParentheses => write!(f, "Mismatched Parentheses"),
            ParseError::DecimalPoint => write!(f, "Decimal Number must have one point"),
            ParseError::DivisionByZero => write!(f, "Division by zero is not defined"),
            ParseError::MissingOperator => write!(f, "Missing Operator between Parentheses"),
        }
    }
}

struct ShuntingYard<'a> {
    infix: &'a str,
}

impl<'a> ShuntingYard<'_> {
    fn precedence(&self, op: String) -> u8 {
        match op.as_str() {
            "^" | "u+" | "u-" => 3,
            "*" | "/" => 2,
            "+" | "-" => 1,
            _ => 0,
        }
    }

    fn is_rt_assoc(&self, op: String) -> bool {
        op == "^" || op == "u+" || op == "u-"
    }

    fn tokenizer(&self) -> Result<Vec<Tokens>, ParseError> {
        if self.infix.trim().is_empty() {
            return Err(ParseError::EmptyExpression);
        }
        let mut out_tks: Vec<Tokens> = Vec::new();
        let mut chars = self.infix.chars().peekable();

        while let Some(&chr) = chars.peek() {
            match chr {
                ' ' | '\n' | '\t' | '\r' => {
                    chars.next();
                }
                '(' => {
                    out_tks.push(Tokens::LeftParen(chr));
                    chars.next();
                }

                ')' => {
                    out_tks.push(Tokens::RightParen);
                    chars.next();
                }

                '^' | '*' | '/' => {
                    out_tks.push(Tokens::Operator(chr));
                    chars.next();
                }
                '+' | '-' => {
                    let is_unary = match out_tks.last() {
                        None => true,
                        Some(Tokens::LeftParen(_))
                        | Some(Tokens::UnaryOperator(_))
                        | Some(Tokens::Operator(_)) => true,
                        _ => false,
                    };

                    if is_unary {
                        while let Some(&u_op) = chars.peek() {
                            match u_op {
                                '+' | '-' => {
                                    out_tks.push(Tokens::UnaryOperator(u_op));
                                    chars.next();
                                }
                                _ => break,
                            }
                        }
                    } else {
                        out_tks.push(Tokens::Operator(chr));
                        chars.next();
                    }
                }

                '0'..='9' | '.' => {
                    let mut num: String = String::new();
                    let mut is_dp: bool = false;
                    while let Some(&n) = chars.peek() {
                        match n {
                            '0'..='9' => {
                                num.push(n);
                                chars.next();
                            }
                            '.' => {
                                if is_dp {
                                    return Err(ParseError::DecimalPoint);
                                }
                                num.push(n);
                                is_dp = true;
                                chars.next();
                            }
                            _ => break,
                        }
                    }
                    out_tks.push(Tokens::Number(num));
                }

                _ => return Err(ParseError::UnknownCharacter(chr)),
            }
        }

        Ok(out_tks)
    }
}

fn main() {
    let expr = ShuntingYard { infix: "(-56+67)" };
    match expr.tokenizer() {
        Ok(tk) => println!("{:?}", tk),
        Err(e) => println!("{:?}", e.to_string()),
    }
}
