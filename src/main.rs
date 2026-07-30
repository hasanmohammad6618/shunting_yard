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

    fn infix_to_postfix(&self) -> Result<Vec<String>, ParseError> {
        let tokens = self.tokenizer()?;
        let mut out_stack: Vec<String> = Vec::new();
        let mut op_stack: Vec<String> = Vec::new();

        for token in tokens {
            match token {
                Tokens::Number(num) => out_stack.push(num),
                Tokens::LeftParen(lp) => {
                    op_stack.push(lp.to_string());
                }
                Tokens::UnaryOperator(u_op) => op_stack.push(format!("u{}", u_op)),
                Tokens::RightParen => {
                    if op_stack.last().unwrap() == "(" {
                        return Err(ParseError::EmptyParentheses);
                    }
                    let mut is_lp: bool = false;
                    while let Some(top) = op_stack.pop() {
                        if top == "(" {
                            is_lp = true;
                            break;
                        } else {
                            out_stack.push(top);
                        }
                    }

                    if !is_lp {
                        return Err(ParseError::MismatchParentheses);
                    }
                }
                Tokens::Operator(op) => {
                    let op_str: String = op.to_string();
                    while let Some(top_op) = op_stack.last() {
                        let top_prec = self.precedence(top_op.clone());
                        let cur_prec = self.precedence(op_str.clone());

                        if top_prec > cur_prec
                            || (top_prec == cur_prec && !self.is_rt_assoc(op_str.clone()))
                        {
                            out_stack.push(op_stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }

                    op_stack.push(op_str);
                }
            }
        }

        while let Some(ro) = op_stack.pop() {
            if ro == "(" {
                return Err(ParseError::MismatchParentheses);
            }
            out_stack.push(ro);
        }

        Ok(out_stack)
    }

    fn evaluate_postfix(&self) -> Result<f64, ParseError> {
        let postfix = self.infix_to_postfix()?;
        let mut stack: Vec<f64> = Vec::new();
        for token in postfix {
            if let Ok(num) = token.parse::<f64>() {
                stack.push(num);
            } else {
                match token.as_str() {
                    "u+" => {
                        let u_pl = stack.pop().unwrap();
                        stack.push(u_pl);
                    }
                    "u-" => {
                        let u_mn = stack.pop().unwrap();
                        stack.push(-u_mn);
                    }
                    "^" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();

                        stack.push(a.powf(b));
                    }
                    "*" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a * b);
                    }
                    "/" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        if b == 0. {
                            return Err(ParseError::DivisionByZero);
                        }
                        stack.push(a / b);
                    }
                    "+" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a + b);
                    }
                    "-" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a - b);
                    }
                    _ => break,
                }
            }
        }
        Ok(stack[0])
    }
}

fn main() {
    let expr = ShuntingYard { infix: "(-56+67)" };
    match expr.evaluate_postfix() {
        Ok(tk) => println!("{:?}", tk),
        Err(e) => println!("{:?}", e.to_string()),
    }
}
