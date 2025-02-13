use crate::definitions::*;
use std::collections::HashMap;

fn evaluate_constant(
    label_idx: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    value: String,
    tipe: &Type,
) -> String {
    match tipe {
        Type::Integer => format!("\tmovq\t${}, %rax\n", value),
        Type::Char => format!("\tmovb\t${}, %al\n", value),
        Type::Boolean => format!("\tmovb\t${}, %al\n", if value == "true" { 1 } else { 0 }),
        Type::Real => {
            let label = request_label(label_idx, rodata, &format!(".double {}", value));
            format!("\tmovsd\tl{}(%rip), %xmm0\n", label)
        }
        Type::Stryng => {
            let label = request_label(label_idx, rodata, &format!(".string \"{}\"", value));
            format!("\tleaq\tl{}(%rip), %rax\n", label)
        }
        _ => panic!("Unsupported type"),
    }
}

// TODO - these should be borrowed, not taken
fn evaluate_type(tipe1: Type, tipe2: Type) -> Type {
    if tipe1 == tipe2 {
        tipe1
    } else if (tipe1 == Type::Integer && tipe2 == Type::Real)
        || (tipe1 == Type::Real && tipe2 == Type::Integer)
    {
        Type::Real
    } else if (tipe1 == Type::Integer && tipe2 == Type::Char)
        || (tipe1 == Type::Char && tipe2 == Type::Integer)
    {
        Type::Char
    } else {
        Type::Undefined
    }
}

fn evaluate_factor(
    factor: &Factor,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    variable_map: &HashMap<String, (isize, Type)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> (String, Type, bool) {
    if let Factor::Constant(unsigned_constant) = factor {
        if let UnsignedConstant::UnsignedInteger(n) = unsigned_constant {
            (format!("{}", n), Type::Integer, true)
        } else if let UnsignedConstant::UnsignedReal(f) = unsigned_constant {
            (format!("{}", f), Type::Real, true)
        } else if let UnsignedConstant::Quote(text) = unsigned_constant {
            (format!("{}", text), Type::Stryng, true)
        } else if let UnsignedConstant::Nil(start) = unsigned_constant {
            report(
                src,
                *start,
                *start + "NIL".len(),
                "Invalid value in expression",
                "error",
            );
            *errors += 1;
            (String::new(), Type::Undefined, false)
        } else if let UnsignedConstant::Char(c) = unsigned_constant {
            (format!("{}", c), Type::Char, true)
        } else {
            panic!("Unrecognized unsigned constant")
        }
    } else if let Factor::Parenthetical(expression) = factor {
        evaluate_expression(
            expression,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        )
    } else if let Factor::NegatedFactor(sub_factor, start, end) = factor {
        let (value, tipe, is_constant) = evaluate_factor(
            sub_factor,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        if is_constant {
            match tipe {
                Type::Boolean => (
                    format!("{}", !value.parse::<bool>().unwrap()),
                    Type::Boolean,
                    true,
                ),
                Type::Integer => (
                    format!("{}", !value.parse::<i64>().unwrap()),
                    Type::Integer,
                    true,
                ),
                _ => {
                    report(src, *start, *end, "Invalid use of NOT", "error");
                    *errors += 1;
                    (String::new(), Type::Undefined, false)
                }
            }
        } else {
            match tipe {
                Type::Boolean => (
                    format!(
                        "{}\
                                        \tsubb\t$1, %al\n\
                                        \tnegb\t%al\n",
                        &value
                    ),
                    Type::Boolean,
                    false,
                ),
                Type::Integer => (
                    format!(
                        "{}\
                        \tnotq\t%rax\n",
                        &value
                    ),
                    Type::Integer,
                    false,
                ),
                _ => {
                    report(src, *start, *end, "Invalid use of NOT", "error");
                    *errors += 1;
                    (String::new(), Type::Undefined, false)
                }
            }
        }
    } else if let Factor::Identifier(name, arguments, start, end) = factor {
        if name == "ORD" {
            if arguments.len() != 1 {
                report(src, *start, *end, "Expected 1 argument", "error");
                *errors += 1;
                return (String::new(), Type::Integer, false);
            }
            let (char_value, tipe, is_constant) = evaluate_expression(
                arguments.get(0).unwrap(),
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if tipe != Type::Char && tipe != Type::Undefined {
                report(src, *start, *end, "Expected char as argument", "error");
                *errors += 1;
            }
            if is_constant {
                (char_value, Type::Integer, true)
            } else {
                // extend char (byte) into full intger, return
                (
                    format!("{}{}", char_value, "\tcbtw\n\tcwtl\n\tcltq\n"),
                    Type::Integer,
                    false,
                )
            }
        } else if name == "CHR" {
            if arguments.len() != 1 {
                report(src, *start, *end, "Expected 1 argument", "error");
                *errors += 1;
                return (String::new(), Type::Char, false);
            }
            let (integer_value, tipe, is_constant) = evaluate_expression(
                arguments.get(0).unwrap(),
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if tipe != Type::Integer {
                report(src, *start, *end, "Expected integer as argument", "error");
                *errors += 1;
            }
            (integer_value, Type::Char, is_constant)
        } else if name == "SQRT" {
            if arguments.len() != 1 {
                report(src, *start, *end, "Expected 1 argument", "error");
                *errors += 1;
                return (String::new(), Type::Real, false);
            }
            let (mut input_value, tipe, is_constant) = evaluate_expression(
                arguments.get(0).unwrap(),
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if is_constant {
                if tipe == Type::Integer || tipe == Type::Real {
                    input_value = input_value.parse::<f64>().unwrap().sqrt().to_string();
                } else {
                    report(
                        src,
                        *start,
                        *end,
                        "Expected integer or real as argument",
                        "error",
                    );
                    *errors += 1;
                }
            } else if tipe == Type::Integer {
                input_value.push_str(
                    "\
                                \tcvtsi2sd %rax, %xmm0\n\
                                \tcall\tsqrt\n",
                );
            } else if tipe == Type::Real {
                input_value.push_str("\tcall\tsqrt\n");
            } else {
                report(
                    src,
                    *start,
                    *end,
                    "Expected integer or real as argument",
                    "error",
                );
                *errors += 1;
            }

            (input_value, Type::Real, is_constant)
        } else if name == "SQR" {
            if arguments.len() != 1 {
                report(src, *start, *end, "Expected 1 argument", "error");
                *errors += 1;
                return (String::new(), Type::Real, false);
            }
            let (mut input_value, tipe, is_constant) = evaluate_expression(
                arguments.get(0).unwrap(),
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if is_constant {
                if tipe == Type::Integer {
                    let int_input_value = input_value.parse::<i64>().unwrap();
                    (
                        format!("{}", int_input_value * int_input_value),
                        Type::Integer,
                        true,
                    )
                } else if tipe == Type::Real {
                    let float_input_value = input_value.parse::<f64>().unwrap();
                    (
                        format!("{}", float_input_value * float_input_value),
                        Type::Real,
                        true,
                    )
                } else {
                    report(
                        src,
                        *start,
                        *end,
                        "Expected integer or real as argument",
                        "error",
                    );
                    *errors += 1;
                    (String::new(), Type::Undefined, false)
                }
            } else if tipe == Type::Integer {
                input_value.push_str(
                    "\
                                \tmovq\t%rax, %rdx\n\
                                \timulq\t%rdx\n",
                );
                (input_value, Type::Integer, false)
            } else if tipe == Type::Real {
                input_value.push_str("\tmulsd\t%xmm0, %xmm0\n");
                (input_value, Type::Real, false)
            } else {
                report(
                    src,
                    *start,
                    *end,
                    "Expected integer or real as argument",
                    "error",
                );
                *errors += 1;
                (String::new(), Type::Undefined, false)
            }
        } else if name == "ABS" {
            if arguments.len() != 1 {
                report(src, *start, *end, "Expected 1 argument", "error");
                *errors += 1;
                return (String::new(), Type::Real, false);
            }
            let (mut input_value, tipe, is_constant) = evaluate_expression(
                arguments.get(0).unwrap(),
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if is_constant {
                if tipe == Type::Integer {
                    let int_input_value = input_value.parse::<i64>().unwrap();
                    (
                        format!(
                            "{}",
                            if int_input_value >= 0 {
                                int_input_value
                            } else {
                                -int_input_value
                            }
                        ),
                        Type::Integer,
                        true,
                    )
                } else if tipe == Type::Real {
                    let float_input_value = input_value.parse::<f64>().unwrap();
                    (
                        format!(
                            "{}",
                            if float_input_value > 0.0 {
                                float_input_value
                            } else {
                                -float_input_value
                            }
                        ),
                        Type::Real,
                        true,
                    )
                } else {
                    report(
                        src,
                        *start,
                        *end,
                        "Expected integer or real as argument",
                        "error",
                    );
                    *errors += 1;
                    (String::new(), Type::Undefined, false)
                }
            } else if tipe == Type::Integer {
                input_value.push_str(
                    "\
                                \tmovq\t%rax, %rdi\n\
                                \tcall\tabs\n",
                );
                (input_value, Type::Integer, false)
            } else if tipe == Type::Real {
                input_value.push_str("\tcall\tfabs\n");
                (input_value, Type::Real, false)
            } else {
                report(
                    src,
                    *start,
                    *end,
                    "Expected integer or real as argument",
                    "error",
                );
                *errors += 1;
                (String::new(), Type::Undefined, false)
            }
        } else if arguments.len() > 0 {
            // TODO - Process function calls
            panic!("Failed to compile function call.")
        } else {
            let mut is_constant = true;
            let (location, tipe) = constant_map
                .get(name)
                .map(|(loc, typ)| (loc.clone(), typ.clone()))
                .unwrap_or_else(|| {
                    is_constant = false;
                    if name == "EOF" {
                        ("eof(%rip)".to_string(), Type::Boolean)
                    } else {
                        let (offset, tipe_holder) = variable_map
                            .get(name)
                            .expect(&format!("Unrecognized identifier: {}", name));
                        (format!("-{}(%rbp)", offset), tipe_holder.clone())
                    }
                });
            if is_constant {
                (location, tipe.clone(), true)
            } else {
                match tipe {
                    Type::Boolean | Type::Char => {
                        (format!("\tmovb\t{}, %al\n", location), tipe.clone(), false)
                    }
                    Type::Integer | Type::Stryng => {
                        (format!("\tmovq\t{}, %rax\n", location), tipe.clone(), false)
                    }
                    Type::Real => (
                        format!("\tmovsd\t{}, %xmm0\n", location),
                        tipe.clone(),
                        false,
                    ),
                    _ => {
                        report(src, *start, *end, "Unsupported type used", "error");
                        *errors += 1;
                        (String::new(), Type::Undefined, false)
                    }
                }
            }
        }
    } else if let Factor::ArrayIndex(name, index, start, end) = factor {
        let (location, array_tipe) = variable_map
            .get(name)
            .expect(&format!("Unrecognized identifier: {}", name));
        if let Type::Array(sub_tipe, _, _) = array_tipe {
            let (index_value, expected_integer) = evaluate_final_expression(
                index,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if expected_integer != Type::Integer {
                report(
                    src,
                    *start,
                    *end,
                    "Arrays must indexed with integer type",
                    "error",
                );
                *errors += 1;
            }
            match **sub_tipe {
                Type::Boolean | Type::Char => (
                    format!(
                        "{}\
                                                    \tmovq\t%rax, %rdx\n\
                                                    \tmovb\t-{}(%rbp, %rdx, 1), %al\n\
                                                    ",
                        index_value, location
                    ),
                    *sub_tipe.clone(),
                    false,
                ),
                Type::Integer | Type::Stryng => (
                    format!(
                        "{}\
                                                            \tmovq\t%rax, %rdx\n\
                                                    \tmovq\t-{}(%rbp, %rdx, 8), %rax\n\
                                                    ",
                        index_value, location
                    ),
                    *sub_tipe.clone(),
                    false,
                ),
                Type::Real => (
                    format!(
                        "{}\
                                                            \tmovsd\t-{}(%rbp, %rax, 8), %xmm0\n\
                                    ",
                        index_value, location
                    ),
                    *sub_tipe.clone(),
                    false,
                ),
                _ => {
                    report(src, *start, *end, "Unsupported type used", "error");
                    *errors += 1;
                    (String::new(), Type::Undefined, false)
                }
            }
        } else if let Type::Stryng = array_tipe {
            let (index_value, expected_integer) = evaluate_final_expression(
                index,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if expected_integer != Type::Integer {
                report(
                    src,
                    *start,
                    *end,
                    "Arrays must indexed with integer type",
                    "error",
                );
                *errors += 1;
            }
            // must be a char since we're indexing in a string
            (
                format!(
                    "{}\
                \taddq\t-{}(%rbp), %rax\n\
                \tdecq\t%rax\n\
                \tmovzbl\t(%rax), %eax\n\
                     ",
                    index_value, location
                ),
                Type::Char,
                false,
            )
        } else {
            report(src, *start, *end, "Not an array or string type", "error");
            *errors += 1;
            (String::new(), Type::Undefined, false)
        }
    } else {
        // TODO - Process lists
        panic!("Failed to compile factor. Possible use of list.")
    }
}

// may use rax, rdx, rcx
fn evaluate_term(
    term: &Term,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    variable_map: &HashMap<String, (isize, Type)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> (String, Type, bool) {
    let (mut value1, mut tipe1, mut is_constant1) = evaluate_factor(
        &term.operands[0].clone(),
        src,
        label_idx,
        errors,
        warnings,
        rodata,
        variable_map,
        constant_map,
    );

    let mut operators_idx = 0;
    while operators_idx < term.operators.len() {
        let operator = term.operators[operators_idx].clone();
        let (mut value2, tipe2, is_constant2) = evaluate_factor(
            &term.operands[operators_idx + 1].clone(),
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        let mut term_tipe = evaluate_type(tipe1.clone(), tipe2.clone());
        if term_tipe == Type::Undefined && tipe1 != Type::Undefined && tipe2 != Type::Undefined {
            report(
                src,
                term.start,
                term.end,
                "Mismatched types in term",
                "error",
            );
            *errors += 1;
            (value1, tipe1, is_constant1) = (String::new(), Type::Undefined, true);
        } else if is_constant1 && is_constant2 {
            let res = match term_tipe {
                Type::Integer if operator == "*" => format!(
                    "{}",
                    value1.parse::<i64>().unwrap() * value2.parse::<i64>().unwrap()
                ),
                Type::Real if operator == "*" => format!(
                    "{}",
                    value1.parse::<f64>().unwrap() * value2.parse::<f64>().unwrap()
                ),
                Type::Integer if operator == "DIV" => format!(
                    "{}",
                    value1.parse::<i64>().unwrap() / value2.parse::<i64>().unwrap()
                ),
                Type::Real if operator == "/" => format!(
                    "{}",
                    value1.parse::<f64>().unwrap() / value2.parse::<f64>().unwrap()
                ),
                Type::Integer if operator == "MOD" => format!(
                    "{}",
                    value1.parse::<i64>().unwrap() % value2.parse::<i64>().unwrap()
                ),
                Type::Real if operator == "MOD" => format!(
                    "{}",
                    value1.parse::<f64>().unwrap() % value2.parse::<f64>().unwrap()
                ),
                Type::Integer if operator == "AND" => format!(
                    "{}",
                    value1.parse::<i64>().unwrap() & value2.parse::<i64>().unwrap()
                ),
                Type::Boolean if operator == "AND" => format!(
                    "{}",
                    value1.parse::<bool>().unwrap() && value2.parse::<bool>().unwrap()
                ),
                Type::Integer if operator == "/" => {
                    report(
                        src,
                        term.start,
                        term.end,
                        "/ is for reals. Did you mean DIV?",
                        "warning",
                    );
                    term_tipe = Type::Real;
                    format!(
                        "{}",
                        value1.parse::<f64>().unwrap() / value2.parse::<f64>().unwrap()
                    )
                }
                Type::Real if operator == "DIV" => {
                    report(
                        src,
                        term.start,
                        term.end,
                        "DIV is for integers. Did you mean /?",
                        "warning",
                    );
                    format!(
                        "{}",
                        value1.parse::<f64>().unwrap() / value2.parse::<f64>().unwrap()
                    )
                }
                Type::Real if operator == "DIV" => panic!("DIV used for reals instead of /"),
                _ => String::new(),
            };
            (value1, tipe1, is_constant1) = (res, term_tipe, true);
        } else {
            let mut out = String::new();
            if is_constant1 {
                value1 = evaluate_constant(label_idx, rodata, value1, &tipe1);
            } else if is_constant2 {
                value2 = evaluate_constant(label_idx, rodata, value2, &tipe2);
            }

            // put first operand in %rax or %xmm0, and second in %rdx or %xmm1
            out.push_str(&value2);
            if term_tipe == Type::Real {
                if tipe1 == Type::Integer {
                    out.push_str("\tsubq\t$8, %rsp\n");
                    out.push_str("\tmovsd\t%xmm0, (%rsp)\n");
                    out.push_str(&value1);
                    out.push_str("\tcvtsi2sd %rax, %xmm0\n");
                    out.push_str("\tmovsd\t(%rsp), %xmm1\n");
                    out.push_str("\taddq\t$8, %rsp\n");
                } else if tipe2 == Type::Integer {
                    out.push_str("\tpushq\t%rax\n");
                    out.push_str(&value1);
                    out.push_str("\tpopq\t%rax\n");
                    out.push_str("\tcvtsi2sd %rax, %xmm1\n");
                } else {
                    out.push_str("\tsubq\t$8, %rsp\n");
                    out.push_str("\tmovsd\t%xmm0, (%rsp)\n");
                    out.push_str(&value1);
                    out.push_str("\tmovsd\t(%rsp), %xmm1\n");
                    out.push_str("\taddq\t$8, %rsp\n");
                }
            } else {
                out.push_str("\tpushq\t%rax\n");
                out.push_str(&value1);
                out.push_str("\tpopq\t%rdx\n");
            }

            // preform the operation
            match operator.as_str() {
                "*" if term_tipe == Type::Integer => out.push_str("\timulq\t%rdx\n"),
                "*" if term_tipe == Type::Real => out.push_str("\tmulsd\t%xmm1, %xmm0\n"),
                "DIV" if term_tipe == Type::Integer => out.push_str(
                    "\
                    \tmovq\t%rdx, %rcx\n\
                                    \tmovq\t$0, %rdx\n\
                                    \tidivq\t%rcx\n",
                ),
                "/" if term_tipe == Type::Integer => {
                    report(
                        src,
                        term.start,
                        term.end,
                        "/ is for reals. Did you mean DIV?",
                        "warning",
                    );
                    term_tipe = Type::Real;
                    out.push_str(
                        "\
                                    \tcvtsi2sd %rax, %xmm0\n\
                                    \tcvtsi2sd %rdx, %xmm1\n\
                                    \tdivsd\t%xmm1, %xmm0\n",
                    )
                }
                "/" if term_tipe == Type::Real => out.push_str("\tdivsd\t%xmm1, %xmm0\n"),
                "DIV" if term_tipe == Type::Real => {
                    report(
                        src,
                        term.start,
                        term.end,
                        "DIV is for integers. Did you mean /?",
                        "warning",
                    );
                    out.push_str("\tdivsd\t%xmm1, %xmm0\n")
                }
                "MOD" if term_tipe == Type::Integer => out.push_str(
                    "\
                                    \tmovq\t%rdx, %rcx\n\
                                    \tmovq\t$0, %rdx\n\
                                    \tidivq\t%rcx\n\
                                    \tmovq\t%rdx, %rax\n",
                ),
                "MOD" if term_tipe == Type::Real => out.push_str(
                    "\
                                        \tcall\tfmod\n",
                ),
                "AND" if term_tipe == Type::Integer => out.push_str("\tandq\t%rdx, %rax\n"),
                "AND" if term_tipe == Type::Boolean => out.push_str("\tandb\t%dl, %al\n"),
                _ if term_tipe == Type::Undefined => {} // already handled
                _ => report(
                    src,
                    term.start,
                    term.end,
                    "Unrecognized operation in term",
                    "error",
                ),
            }
            (value1, tipe1, is_constant1) = (out, term_tipe, false);
        }
        operators_idx += 1;
    }
    (value1, tipe1, is_constant1)
}

fn evaluate_simple_expression(
    simple_expression: &SimpleExpression,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    variable_map: &HashMap<String, (isize, Type)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> (String, Type, bool) {
    let (mut value1, mut tipe1, mut is_constant1) = evaluate_term(
        &simple_expression.operands[0].clone(),
        src,
        label_idx,
        errors,
        warnings,
        rodata,
        variable_map,
        constant_map,
    );

    if !simple_expression.positive {
        if is_constant1 {
            value1 = format!("-{}", value1);
        } else {
            match tipe1 {
                Type::Integer => value1.push_str("\tnegq\t%rax\n"),
                Type::Real => value1.push_str("\tmovq\t$0x8000000000000000, %rax\n\tmovq\t%rax, %xmm2\n\txorpd\t%xmm2, %xmm0\n"),
                Type::Undefined => {},
                _ => {
                    report(src, simple_expression.start, simple_expression.end, "Unrecognized attempt to negate first term", "error");
                    *errors += 1;
                },
            }
        }
    }

    let mut operators_idx = 0;
    while operators_idx < simple_expression.operators.len() {
        let operator = simple_expression.operators[operators_idx].clone();
        let (mut value2, tipe2, is_constant2) = evaluate_term(
            &simple_expression.operands[operators_idx + 1].clone(),
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        let simple_expression_tipe = evaluate_type(tipe1.clone(), tipe2.clone());
        if simple_expression_tipe == Type::Undefined
            && tipe1 != Type::Undefined
            && tipe2 != Type::Undefined
        {
            report(
                src,
                simple_expression.start,
                simple_expression.end,
                "Mismatched types in term",
                "error",
            );
            *errors += 1;
            (value1, tipe1, is_constant1) = (String::new(), Type::Undefined, true);
        } else if is_constant1 && is_constant2 {
            // evaluate constant
            let res = match simple_expression_tipe {
                Type::Integer if operator == "+" => format!(
                    "{}",
                    value1.parse::<i64>().unwrap() + value2.parse::<i64>().unwrap()
                ),
                Type::Real if operator == "+" => format!(
                    "{}",
                    value1.parse::<f64>().unwrap() + value2.parse::<f64>().unwrap()
                ),
                Type::Stryng if operator == "+" => format!("{}{}", value1, value2),
                Type::Integer if operator == "-" => format!(
                    "{}",
                    value1.parse::<i64>().unwrap() - value2.parse::<i64>().unwrap()
                ),
                Type::Real if operator == "-" => format!(
                    "{}",
                    value1.parse::<f64>().unwrap() - value2.parse::<f64>().unwrap()
                ),
                Type::Integer if operator == "OR" => format!(
                    "{}",
                    value1.parse::<i64>().unwrap() | value2.parse::<i64>().unwrap()
                ),
                Type::Boolean if operator == "OR" => format!(
                    "{}",
                    value1.parse::<bool>().unwrap() || value2.parse::<bool>().unwrap()
                ),
                _ => String::new(),
            };
            (value1, tipe1, is_constant1) = (res, simple_expression_tipe, true);
        } else {
            let mut out = String::new();
            if is_constant1 {
                value1 = evaluate_constant(label_idx, rodata, value1, &tipe1);
            } else if is_constant2 {
                value2 = evaluate_constant(label_idx, rodata, value2, &tipe2);
            }

            // put first operand in %rax or %xmm0, and second in %rdx or %xmm1
            out.push_str(&value2);
            if simple_expression_tipe == Type::Real {
                if tipe1 == Type::Integer {
                    out.push_str("\tpushq\t%rax\n");
                    out.push_str(&value1);
                    out.push_str("\tpopq\t%rax\n");
                    out.push_str("\tcvtsi2sd %rax, %xmm1\n");
                } else if tipe2 == Type::Integer {
                    out.push_str(&value1);
                    out.push_str("\tcvtsi2sd %rax, %xmm1\n");
                } else {
                    out.push_str("\tsubq\t$8, %rsp\n");
                    out.push_str("\tmovsd\t%xmm0, (%rsp)\n");
                    out.push_str(&value1);
                    out.push_str("\tmovsd\t(%rsp), %xmm1\n");
                    out.push_str("\taddq\t$8, %rsp\n");
                }
            } else {
                out.push_str("\tpushq\t%rax\n");
                out.push_str(&value1);
                out.push_str("\tpopq\t%rdx\n");
            }

            // preform the operation
            match operator.as_str() {
                "+" if simple_expression_tipe == Type::Integer => {
                    out.push_str("\taddq\t%rdx, %rax\n")
                }
                "+" if simple_expression_tipe == Type::Real => {
                    out.push_str("\taddsd\t%xmm1, %xmm0\n")
                }
                "+" if simple_expression_tipe == Type::Char => out.push_str("\taddb\t%dl, %al\n"),
                // TODO - Implement + for type String
                "-" if simple_expression_tipe == Type::Integer => {
                    out.push_str("\tsubq\t%rdx, %rax\n")
                }
                "-" if simple_expression_tipe == Type::Real => {
                    out.push_str("\tsubsd\t%xmm1, %xmm0\n")
                }
                "-" if simple_expression_tipe == Type::Char => out.push_str("\tsubb\t%dl, %al\n"),
                "OR" if simple_expression_tipe == Type::Integer => {
                    out.push_str("\torq\t%rdx, %rax\n")
                }
                "OR" if simple_expression_tipe == Type::Boolean => {
                    out.push_str("\torb\t%dl, %al\n")
                }
                _ if simple_expression_tipe == Type::Undefined => {} // already handled
                _ => {
                    report(
                        src,
                        simple_expression.start,
                        simple_expression.end,
                        "Unrecognized operation",
                        "error",
                    );
                    *errors += 1;
                }
            }
            (value1, tipe1, is_constant1) = (out, simple_expression_tipe, false);
        }
        operators_idx += 1;
    }
    (value1, tipe1, is_constant1)
}

// returns code to evaluate an expression using the registers %rax and %rdx
// expression ending up in %rax
fn evaluate_expression(
    expression: &Expression,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    variable_map: &HashMap<String, (isize, Type)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> (String, Type, bool) {
    let (mut value1, tipe1, is_constant1) = evaluate_simple_expression(
        &expression.operand1,
        src,
        label_idx,
        errors,
        warnings,
        rodata,
        variable_map,
        constant_map,
    );

    if &expression.operator != "NONE" {
        let (mut value2, tipe2, is_constant2) = evaluate_simple_expression(
            &expression.operand2,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        let expression_tipe = evaluate_type(tipe1.clone(), tipe2.clone());

        if is_constant1 && is_constant2 {
            let out = if expression_tipe == Type::Real {
                match expression.operator.as_str() {
                    "<" => format!(
                        "{}",
                        value1.parse::<f64>().unwrap() < value2.parse::<f64>().unwrap()
                    ),
                    "<=" => format!(
                        "{}",
                        value1.parse::<f64>().unwrap() <= value2.parse::<f64>().unwrap()
                    ),
                    "=" => format!(
                        "{}",
                        value1.parse::<f64>().unwrap() == value2.parse::<f64>().unwrap()
                    ),
                    "<>" => format!(
                        "{}",
                        value1.parse::<f64>().unwrap() != value2.parse::<f64>().unwrap()
                    ),
                    ">" => format!(
                        "{}",
                        value1.parse::<f64>().unwrap() > value2.parse::<f64>().unwrap()
                    ),
                    ">=" => format!(
                        "{}",
                        value1.parse::<f64>().unwrap() >= value2.parse::<f64>().unwrap()
                    ),
                    _ => {
                        report(
                            src,
                            expression.start,
                            expression.end,
                            "Unrecognized operator",
                            "error",
                        );
                        *errors += 1;
                        String::new()
                    }
                }
            } else if expression_tipe == Type::Integer {
                match expression.operator.as_str() {
                    "<" => format!(
                        "{}",
                        value1.parse::<i64>().unwrap() < value2.parse::<i64>().unwrap()
                    ),
                    "<=" => format!(
                        "{}",
                        value1.parse::<i64>().unwrap() <= value2.parse::<i64>().unwrap()
                    ),
                    "=" => format!(
                        "{}",
                        value1.parse::<i64>().unwrap() == value2.parse::<i64>().unwrap()
                    ),
                    "<>" => format!(
                        "{}",
                        value1.parse::<i64>().unwrap() != value2.parse::<i64>().unwrap()
                    ),
                    ">" => format!(
                        "{}",
                        value1.parse::<i64>().unwrap() > value2.parse::<i64>().unwrap()
                    ),
                    ">=" => format!(
                        "{}",
                        value1.parse::<i64>().unwrap() >= value2.parse::<i64>().unwrap()
                    ),
                    _ => {
                        report(
                            src,
                            expression.start,
                            expression.end,
                            "Unrecognized operator",
                            "error",
                        );
                        *errors += 1;
                        String::new()
                    }
                }
            } else if expression_tipe == Type::Char {
                match expression.operator.as_str() {
                    "<" => format!(
                        "{}",
                        value1.parse::<u8>().unwrap() < value2.parse::<u8>().unwrap()
                    ),
                    "<=" => format!(
                        "{}",
                        value1.parse::<u8>().unwrap() <= value2.parse::<u8>().unwrap()
                    ),
                    "=" => format!(
                        "{}",
                        value1.parse::<u8>().unwrap() == value2.parse::<u8>().unwrap()
                    ),
                    "<>" => format!(
                        "{}",
                        value1.parse::<u8>().unwrap() != value2.parse::<u8>().unwrap()
                    ),
                    ">" => format!(
                        "{}",
                        value1.parse::<u8>().unwrap() > value2.parse::<u8>().unwrap()
                    ),
                    ">=" => format!(
                        "{}",
                        value1.parse::<u8>().unwrap() >= value2.parse::<u8>().unwrap()
                    ),
                    _ => {
                        report(
                            src,
                            expression.start,
                            expression.end,
                            "Unrecognized operator",
                            "error",
                        );
                        *errors += 1;
                        String::new()
                    }
                }
            } else {
                panic!("Invalid type"); // TODO - Not a meaningful error
            };
            (out, Type::Boolean, true)
        } else {
            let mut out = String::new();
            if is_constant1 {
                value1 = evaluate_constant(label_idx, rodata, value1, &tipe1);
            } else if is_constant2 {
                value2 = evaluate_constant(label_idx, rodata, value2, &tipe2);
            }
            out.push_str(&value1);

            if tipe1 == Type::Real {
                out.push_str("\tsubq\t$8, %rsp\n\tmovsd\t%xmm0, (%rsp)\n");
            } else {
                out.push_str("\tpushq\t%rax\n");
            }

            if expression_tipe == Type::Undefined
                && tipe1 != Type::Undefined
                && tipe2 != Type::Undefined
            {
                report(
                    src,
                    expression.start,
                    expression.end,
                    "Mismatched types in expression",
                    "error",
                );
                *errors += 1;
            }
            out.push_str(&value2);
            if tipe1 == Type::Real {
                out.push_str(
                    "\
                        \tmovsd\t(%rsp), %xmm1\n\
                    \taddq\t$8, %rsp\n",
                );
                if tipe2 == Type::Integer {
                    out.push_str("\tcvtsi2sd %rax, %xmm0\n");
                }
                out.push_str(
                    "\txorb\t%al, %al\n\
                    \tucomisd\t%xmm0, %xmm1\n",
                );
            } else if tipe2 == Type::Real {
                out.push_str(
                    "\
                        \tpopq\t%rax\n\
                        \tcvtsi2sd %rax, %xmm1\n\
                        \txorb\t%al, %al\n\
                        \tucomisd\t%xmm0, %xmm1\n\
                            ",
                );
            } else if expression_tipe == Type::Char {
                out.push_str(
                    "\
                        \tpopq\t%rdx\n\
                        \tmovb\t%al, %cl\n\
                        \txorb\t%al, %al\n\
                        \tcmpb\t%cl, %dl\n",
                );
            } else if expression_tipe == Type::Integer {
                out.push_str(
                    "\
                        \tpopq\t%rdx\n\
                        \tmovq\t%rax, %rcx\n\
                        \txorb\t%al, %al\n\
                        \tcmpq\t%rcx, %rdx\n",
                );
            }
            let jump_instruction = if expression_tipe == Type::Real {
                match expression.operator.as_str() {
                    "<" => format!("\tjae\tl{}\n", *label_idx),
                    "<=" => format!("\tja\tl{}\n", *label_idx),
                    "=" => format!("\tjne\tl{}\n", *label_idx),
                    "<>" => format!("\tje\tl{}\n", *label_idx),
                    ">=" => format!("\tjb\tl{}\n", *label_idx),
                    ">" => format!("\tjbe\tl{}\n", *label_idx),
                    "IN" => format!("\tjmp\tl{} # Error: IN not implemented \n", *label_idx),
                    _ => {
                        report(
                            src,
                            expression.start,
                            expression.end,
                            "Unrecognized operator",
                            "error",
                        );
                        *errors += 1;
                        String::new()
                    }
                }
            } else {
                match expression.operator.as_str() {
                    "<" => format!("\tjge\tl{}\n", *label_idx),
                    "<=" => format!("\tjg\tl{}\n", *label_idx),
                    "=" => format!("\tjne\tl{}\n", *label_idx),
                    "<>" => format!("\tje\tl{}\n", *label_idx),
                    ">=" => format!("\tjl\tl{}\n", *label_idx),
                    ">" => format!("\tjle\tl{}\n", *label_idx),
                    "IN" => format!("\tjmp\tl{} # Error: IN not implemented \n", *label_idx),
                    _ => {
                        report(
                            src,
                            expression.start,
                            expression.end,
                            "Unrecognized operator",
                            "error",
                        );
                        *errors += 1;
                        String::new()
                    }
                }
            };
            out.push_str(&jump_instruction);
            out.push_str(&format!("\tincb\t%al\nl{}:\n", *label_idx));
            *label_idx += 1;
            // TODO - Implement IN operator
            (out, Type::Boolean, false)
        }
    } else {
        (value1, tipe1, is_constant1)
    }
}

fn evaluate_final_expression(
    expression: &Expression,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    variable_map: &HashMap<String, (isize, Type)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> (String, Type) {
    let (value, tipe, is_constant) = evaluate_expression(
        expression,
        src,
        label_idx,
        errors,
        warnings,
        rodata,
        variable_map,
        constant_map,
    );
    if is_constant {
        (evaluate_constant(label_idx, rodata, value, &tipe), tipe)
    } else {
        (value, tipe)
    }
}

fn process_block(
    code: &Block,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
) -> String {
    let mut out = String::new();

    let mut stack_offset = 0;
    let constant_map = get_constant_map(&code.constants, src, label_idx, errors, warnings);
    for variable in &code.local_variables {
        let tipe = convert_supertype_to_type(
            &variable.tipe,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            &constant_map,
        );
        stack_offset += get_size(&tipe);
    }

    if stack_offset % 16 > 0 {
        stack_offset = (stack_offset / 16 + 1) * 16;
    }

    let variable_map = get_variable_map(
        &code.local_variables,
        src,
        label_idx,
        errors,
        warnings,
        rodata,
        &constant_map,
    );

    if stack_offset > 0 {
        out.push_str(&format!("\tsubq\t${}, %rsp\n", stack_offset));
    }
    if let Statement::StatementList(ref statements) = &code.body {
        for statement in statements {
            out.push_str(&process_statement(
                statement,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                &variable_map,
                &constant_map,
            ));
        }
    } else {
        panic!("Block type must have a StatementList as the body.");
    }
    if stack_offset > 0 {
        out.push_str(&format!("\taddq\t${}, %rsp\n", stack_offset));
    }
    out
}

// returns size of type
fn get_size(tipe: &Type) -> usize {
    match tipe {
        Type::Integer => 8,
        Type::Boolean => 1,
        Type::Real => 8,
        Type::Char => 1,
        Type::Stryng => 8,
        Type::Array(sub_tipe, start_idx, end_idx) => {
            ((end_idx - start_idx + 1) as usize) * get_size(sub_tipe)
        }
        _ => panic!("Failed to evaluate type size."),
    }
}

fn convert_supertype_to_type(
    super_type: &SuperType,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> Type {
    match super_type {
        SuperType::Integer => Type::Integer,
        SuperType::Boolean => Type::Boolean,
        SuperType::Real => Type::Real,
        SuperType::Char => Type::Char,
        SuperType::Stryng => Type::Stryng,
        SuperType::Text => Type::Text,
        SuperType::Array(element_type, start_expr, end_expr) => {
            let (start_index, _, _) = evaluate_expression(
                start_expr,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                &HashMap::new(),
                constant_map,
            );
            let (end_index, _, _) = evaluate_expression(
                end_expr,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                &HashMap::new(),
                constant_map,
            );
            let converted_element_type = convert_supertype_to_type(
                element_type,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                constant_map,
            );
            // TODO - Add useful error messages (each must be CONSTANT and INTEGERS)
            Type::Array(
                Box::new(converted_element_type),
                start_index.parse::<isize>().expect("Invalid start idx"),
                end_index.parse::<isize>().expect("Invalid end idx"),
            )
        }
    }
}

// returns map of symbol names to their %rbp offsets/types
fn get_variable_map(
    variables: &Vec<Variable>,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> HashMap<String, (isize, Type)> {
    let mut result = HashMap::new();

    let mut stack_offset: isize = 0;
    for variable in variables {
        let tipe = convert_supertype_to_type(
            &variable.tipe,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            constant_map,
        );
        let size = get_size(&tipe);
        stack_offset += size as isize;

        // because pascal arrays are weird...
        let stack_offset_offset = if let Type::Array(_, start_idx, _) = &tipe {
            *start_idx
        } else {
            0
        };
        result.insert(
            variable.name.clone(),
            (stack_offset + stack_offset_offset, tipe.clone()),
        );
    }
    result
}

fn get_constant_map(
    constants: &Vec<Constant>,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
) -> HashMap<String, (String, Type)> {
    let mut result = HashMap::new();

    // generic constants
    result.insert("TRUE".to_string(), ("true".to_string(), Type::Boolean));
    result.insert("FALSE".to_string(), ("false".to_string(), Type::Boolean));
    result.insert(
        "MAXINT".to_string(),
        (format!("${}", 2_u64.pow(63) - 1), Type::Integer),
    );

    // user defined constants
    for constant in constants {
        let (value, tipe, _) = evaluate_expression(
            &constant.value,
            src,
            label_idx,
            errors,
            warnings,
            &mut Vec::new(),
            &HashMap::new(),
            &result,
        );
        result.insert(constant.name.clone(), (value, tipe));
    }

    result
}

fn process_statement(
    code: &Statement,
    src: &str,
    label_idx: &mut u32,
    errors: &mut u32,
    warnings: &mut u32,
    rodata: &mut Vec<(u32, String)>,
    variable_map: &HashMap<String, (isize, Type)>,
    constant_map: &HashMap<String, (String, Type)>,
) -> String {
    let mut out = String::new();
    if let Statement::Assignment(name, expression, start, end) = code {
        let (value, tipe1) = evaluate_final_expression(
            expression,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        out.push_str(&value);
        let (location, tipe2) = variable_map
            .get(name)
            .expect(&format!("Unrecognized identifier: {}", name));
        if tipe1 != *tipe2
            && !(tipe1 == Type::Integer && *tipe2 == Type::Real)
            && !(tipe1 == Type::Char && *tipe2 == Type::Stryng)
        {
            report(src, *start, *end, "Mismatched types", "error");
            *errors += 1;
        }
        out.push_str(&match tipe2 {
            Type::Char | Type::Boolean => format!("\tmovb\t%al, -{}(%rbp)\n", location),
            Type::Stryng if tipe1 == Type::Stryng => format!("\tmovq\t%rax, -{}(%rbp)\n", location),
            Type::Stryng if tipe1 == Type::Char => format!(
                "\tmovb\t%al, -{}(%rbp)\n\tmovb\t$0, -{}(%rbp)\n",
                location,
                location + 1
            ),
            Type::Integer => format!("\tmovq\t%rax, -{}(%rbp)\n", location),
            Type::Real if tipe1 == Type::Real => format!("\tmovq\t%xmm0, -{}(%rbp)\n", location),
            Type::Real if tipe1 == Type::Integer => format!(
                "\tcvtsi2sd %rax, %xmm0\n\tmovq\t%xmm0, -{}(%rbp)\n",
                location
            ),
            _ => panic!("Unsupported type used in assignment"),
        });
    } else if let Statement::ElementAssignment(name, index, expression, start, end) = code {
        let (index_value, expected_integer) = evaluate_final_expression(
            index,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        if expected_integer != Type::Integer {
            report(
                src,
                *start,
                *end,
                "Expected integer to access array element",
                "error",
            );
            *errors += 1;
        }
        let (location, arr_tipe) = variable_map
            .get(name)
            .expect(&format!("Unrecognized identifier: {}", name));
        if let Type::Array(sub_tipe, _, _) = arr_tipe {
            out.push_str(&index_value);
            out.push_str("\tpushq\t%rax\n");
            let (value, tipe) = evaluate_final_expression(
                expression,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            );
            if **sub_tipe != tipe && !(**sub_tipe == Type::Real && tipe == Type::Integer) {
                report(src, *start, *end, "Mismatched types", "error");
                *errors += 1;
            }
            out.push_str(&value);
            out.push_str("\tpopq\t%rdx\n");
            out.push_str(&match **sub_tipe {
                Type::Char | Type::Boolean => {
                    format!("\tmovb\t%al, -{}(%rbp, %rdx, 1)\n", location)
                }
                Type::Stryng | Type::Integer => {
                    format!("\tmovq\t%rax, -{}(%rbp, %rdx, 8)\n", location)
                }
                Type::Real if tipe == Type::Real => {
                    format!("\tmovq\t%xmm0, -{}(%rbp, %rdx, 8)\n", location)
                }
                Type::Real if tipe == Type::Integer => format!(
                    "\tcvtsi2sd %rax, %xmm0\n\tmovq\t%xmm0, -{}(%rbp, %rdx, 8)\n",
                    location
                ),
                _ => panic!("Unsupported type used in assignment"),
            });
        } else {
            report(
                src,
                *start,
                *end,
                "Identifier does not belong to an array",
                "error",
            );
            *errors += 1;
        }
    } else if let Statement::ProcedureCall(name, arguments, start, end) = code {
        if name == "WRITELN" || name == "WRITE" {
            if name == "WRITELN" && arguments.len() == 0 {
                let label = request_label(label_idx, rodata, ".string \"\\n\"");
                out.push_str(&format!(
                    "\
                            \tleaq\tl{}(%rip), %rdi\n\
                            \tmovq\t$0, %rax\n\
                            \tcall\tprintf\n",
                    label
                ));
            }
            for i in 0..arguments.len() {
                let (mut value, tipe, is_constant) = evaluate_expression(
                    &arguments[i],
                    src,
                    label_idx,
                    errors,
                    warnings,
                    rodata,
                    variable_map,
                    constant_map,
                );
                if is_constant && tipe != Type::Stryng {
                    value = evaluate_constant(label_idx, rodata, value, &tipe);
                }
                if !(is_constant && tipe == Type::Stryng) {
                    out.push_str(&value);
                }
                let new_line = name == "WRITELN" && i == arguments.len() - 1;
                match tipe {
                    Type::Integer => {
                        let label = request_label(
                            label_idx,
                            rodata,
                            &format!(".string \"{}\"", if new_line { "%ld\\n" } else { "%ld" }),
                        );
                        out.push_str(&format!(
                            "\
                            \tmovq\t%rax, %rsi\n\
                            \tleaq\tl{}(%rip), %rdi\n\
                            \tmovq\t$0, %rax\n\
                            \tcall\tprintf\n",
                            label
                        ));
                    }
                    Type::Real => {
                        let label = request_label(
                            label_idx,
                            rodata,
                            &format!(".string \"{}\"", if new_line { "%lf\\n" } else { "%lf" }),
                        );
                        // floats should already be in %xmm0
                        out.push_str(&format!(
                            "\
                            \tleaq\tl{}(%rip), %rdi\n\
                            \tmovq\t$1, %rax\n\
                            \tcall\tprintf\n",
                            label
                        ));
                    }
                    Type::Stryng if !is_constant => {
                        let label = request_label(
                            label_idx,
                            rodata,
                            &format!(".string \"{}\"", if new_line { "%s\\n" } else { "%s" }),
                        );
                        out.push_str(&format!(
                            "\
                            \tmovq\t%rax, %rsi\n\
                            \tleaq\tl{}(%rip), %rdi\n\
                            \tmovq\t$0, %rax\n\
                            \tcall\tprintf\n",
                            label
                        ));
                    }
                    Type::Stryng if is_constant => {
                        let label = request_label(
                            label_idx,
                            rodata,
                            &format!(
                                ".string \"{}\"",
                                if new_line {
                                    format!("{}\\n", &value)
                                } else {
                                    format!("{}", &value)
                                }
                            ),
                        );
                        out.push_str(&format!(
                            "\
                            \tleaq\tl{}(%rip), %rdi\n\
                            \tmovq\t$0, %rax\n\
                            \tcall\tprintf\n",
                            label
                        ));
                    }
                    Type::Char => {
                        let label = request_label(
                            label_idx,
                            rodata,
                            &format!(".string \"{}\"", if new_line { "%c\\n" } else { "%c" }),
                        );
                        out.push_str(&format!(
                            "\
                            \tmovb\t%al, %sil\n\
                            \tleaq\tl{}(%rip), %rdi\n\
                            \tmovq\t$0, %rax\n\
                            \tcall\tprintf\n",
                            label
                        ));
                    }
                    Type::Boolean => {
                        let label_false = request_label(
                            label_idx,
                            rodata,
                            &format!(
                                ".string \"{}\"",
                                if new_line { "FALSE\\n" } else { "FALSE" }
                            ),
                        );
                        let label_true = request_label(
                            label_idx,
                            rodata,
                            &format!(".string \"{}\"", if new_line { "TRUE\\n" } else { "TRUE" }),
                        );
                        let jmp_label = *label_idx;
                        *label_idx += 1;
                        out.push_str(&format!(
                            "\
                            \tleaq\tl{}(%rip), %rdi\n\
                            \ttestb\t%al, %al\n\
                            \tje\tl{}\n\
                            \tleaq\tl{}(%rip), %rdi\n\
                            l{}:\n\
                            \tmovq\t$0, %rax\n\
                            \tcall\tprintf\n\
                            ",
                            label_false, jmp_label, label_true, jmp_label
                        ));
                    }
                    Type::Undefined => {} // Already handled
                    _ => {
                        report(
                            src,
                            *start,
                            *end,
                            "Print function not defined for all types in call",
                            "error",
                        );
                        *errors += 1;
                    }
                }
            }
        } else {
            out.push_str(&format!("\t# Failed to compile call to: {}", name));
        }
    } else if let Statement::ReadCall(vars, start, end) = code {
        for var in vars {
            let (location, tipe) = variable_map
                .get(var)
                .expect(&format!("Unrecognized identifier: {}", var));
            match tipe {
                Type::Char => {
                    let l1 = *label_idx;
                    *label_idx += 1;
                    let l2 = *label_idx;
                    *label_idx += 1;
                    out.push_str(&format!(
                        "\
                        l{}:
                        \tcall\tgetchar\n\
                        \tcmpl\t$-1, %eax\n\
                        \tjne\tl{}\n\
                        \tmovl\t$1, eof(%rip)\n\
                        l{}:\n\
                        \tcmpb\t$10, %al\n\
                        \tje\tl{}
                        \tmovb\t%al, -{}(%rbp)\n\
                            ",
                        l1, l2, l2, l1, location
                    ));
                }
                Type::Integer => {
                    let label = request_label(label_idx, rodata, ".string \"%ld\"");
                    out.push_str(&format!(
                        "\
                        \tleaq\t-{}(%rbp), %rsi\n\
                        \tleaq\tl{}(%rip), %rdi\n\
                        \tmovq\t$0, %rax\n\
                        \tcall\tscanf\n\
                            ",
                        location, label
                    ));
                }
                Type::Real => {
                    let label = request_label(label_idx, rodata, ".string \"%lf\"");
                    out.push_str(&format!(
                        "\
                        \tleaq\t-{}(%rbp), %rsi\n\
                        \tleaq\tl{}(%rip), %rdi\n\
                        \tmovq\t$0, %rax\n\
                        \tcall\tscanf\n\
                            ",
                        location, label
                    ));
                }
                // calloc 256 bytes for string input, then take string input
                Type::Stryng => {
                    let label = request_label(label_idx, rodata, ".string \" %[^\\n]s\"");
                    out.push_str(&format!(
                        "\
                        \tmovq\t$256, %rdi\n\
                        \tmovq\t$1, %rsi\n\
                        \tcall\tcalloc\n\
                        \tmovq\t%rax, -{}(%rbp)\n\
                        \tmovq\t%rax, %rsi\n\
                        \tleaq\tl{}(%rip), %rdi\n\
                        \tmovq\t$0, %rax\n\
                        \tcall\tscanf\n\
                            ",
                        location, label
                    ));
                }
                _ => {
                    report(src, *start, *end, "Unsupported type in read call", "error");
                    *errors += 1;
                }
            }
        }
    } else if let Statement::StatementList(statements) = code {
        for statement in statements {
            out.push_str(&process_statement(
                statement,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            ));
        }
    } else if let Statement::IfStatement(
        condition,
        true_body,
        false_body,
        condition_start,
        condition_end,
    ) = code
    {
        let has_else = !matches!(**false_body, Statement::DoNothing);
        let (value, tipe) = evaluate_final_expression(
            condition,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        if tipe != Type::Boolean && tipe != Type::Undefined {
            report(
                src,
                *condition_start,
                *condition_end,
                "Condition must be a boolean type",
                "error",
            );
            *errors += 1;
        }
        out.push_str(&value);
        let l1 = *label_idx;
        *label_idx += 1;
        let mut l2: u32 = *label_idx; // this is only for if there's an else statement
        out.push_str(&format!(
            "\
                     \ttestb\t%al, %al\n\
                     \tje\tl{}\n\
                     ",
            l1
        ));
        out.push_str(&process_statement(
            true_body,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        ));
        if has_else {
            l2 = *label_idx;
            *label_idx += 1;
            out.push_str(&format!("\tjmp\tl{}\n", l2));
        }

        out.push_str(&format!("l{}:\n", l1));
        if has_else {
            out.push_str(&process_statement(
                false_body,
                src,
                label_idx,
                errors,
                warnings,
                rodata,
                variable_map,
                constant_map,
            ));
            out.push_str(&format!("l{}:\n", l2));
        }
    } else if let Statement::WhileLoop(condition, body, condition_start, condition_end) = code {
        let l1 = *label_idx;
        *label_idx += 1;
        out.push_str(&format!("l{}:\n", l1));
        let (value, tipe) = evaluate_final_expression(
            condition,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        if tipe != Type::Boolean {
            report(
                src,
                *condition_start,
                *condition_end,
                "Condition must be a boolean type",
                "error",
            );
            *errors += 1;
        }
        out.push_str(&value);
        let l2 = *label_idx;
        *label_idx += 1;
        out.push_str(&format!(
            "\
                    \ttestb\t%al, %al\n\
                    \tje\tl{}\n",
            l2
        ));
        out.push_str(&process_statement(
            body,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        ));
        out.push_str(&format!(
            "\
                    \tjmp\tl{}\n\
                    l{}:\n\
                    ",
            l1, l2
        ));
    } else if let Statement::RepeatLoop(condition, body, condition_start, condition_end) = code {
        let l1 = *label_idx;
        *label_idx += 1;
        let (value, tipe) = evaluate_final_expression(
            condition,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        if tipe != Type::Boolean {
            report(
                src,
                *condition_start,
                *condition_end,
                "Condition must be a boolean type",
                "error",
            );
            *errors += 1;
        }
        out.push_str(&format!("l{}:\n", l1));
        out.push_str(&process_statement(
            body,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        ));
        out.push_str(&value);
        out.push_str(&format!(
            "\
                    \ttestb\t%al, %al\n\
                    \tje\tl{}\n",
            l1
        ));
    } else if let Statement::ForLoop(
        name,
        name_start,
        name_end,
        start,
        end,
        range_start,
        range_end,
        ascending,
        body,
    ) = code
    {
        let (location, tipe) = variable_map
            .get(name)
            .expect(&format!("Unrecognized identifier: {}", name));
        if *tipe != Type::Integer {
            report(
                src,
                *name_start,
                *name_end,
                "For loop iterator must be integer type",
                "error",
            );
            *errors += 1;
        }
        let (start_value, start_tipe) = evaluate_final_expression(
            start,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        let (end_value, end_tipe) = evaluate_final_expression(
            end,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        );
        if start_tipe != Type::Integer || end_tipe != Type::Integer {
            report(
                src,
                *range_start,
                *range_end,
                "For loop range must consist of integers",
                "error",
            );
            *errors += 1;
        }
        out.push_str(&start_value);
        out.push_str(&format!("\tmovq\t%rax, -{}(%rbp)\n", location));
        out.push_str(&end_value);

        // since for loop ranges are inclusive, we do this to simplify code
        if *ascending {
            out.push_str("\tincq\t%rax\n");
        } else {
            out.push_str("\tdecq\t%rax\n");
        }

        out.push_str("\tpushq\t$0\n"); // to keep stack 16-byte aligned
        out.push_str("\tpushq\t%rax\n");
        let l1 = *label_idx;
        *label_idx += 1;
        let l2 = *label_idx;
        *label_idx += 1;
        out.push_str(&format!(
            "l{}:\n\
                            \tmovq\t(%rsp), %rax\n\
                            \tmovq\t-{}(%rbp), %rdx\n\
                            \tcmpq\t%rax, %rdx\n\
                            \tje\tl{}\n",
            l1, location, l2
        ));
        out.push_str(&process_statement(
            body,
            src,
            label_idx,
            errors,
            warnings,
            rodata,
            variable_map,
            constant_map,
        ));
        if *ascending {
            out.push_str(&format!("\tincq\t-{}(%rbp)\n", location));
        } else {
            out.push_str(&format!("\tdecq\t-{}(%rbp)\n", location));
        }
        out.push_str(&format!(
            "\tjmp\tl{}\n\
                            l{}:\n\
                            \taddq\t$16, %rsp\n",
            l1, l2
        ));
    }
    out
}

fn request_label(label_idx: &mut u32, rodata: &mut Vec<(u32, String)>, value: &str) -> u32 {
    for (label, instance_value) in &mut *rodata {
        if instance_value.as_str() == value {
            return *label;
        }
    }
    rodata.push((*label_idx, value.to_string()));
    *label_idx += 1;
    *label_idx - 1
}

// returns a string of the resulting x86-64 code
pub fn compile(code: Program, src: &str) -> (String, u32, u32) {
    let mut x86_64 = String::new();

    let mut rodata: Vec<(u32, String)> = Vec::new();
    let mut label_idx = 0;

    let mut errors = 0;
    let mut warnings = 0;
    let body = process_block(
        &code.body,
        src,
        &mut label_idx,
        &mut errors,
        &mut warnings,
        &mut rodata,
    );

    // TODO - This is where globals will go, for now just eof
    if body.contains("eof") {
        x86_64.push_str(".section .data\neof:\n\t.int 0\n");
    }

    if rodata.len() > 0 {
        x86_64.push_str(".section .rodata\n");
        for (label, value) in rodata {
            x86_64.push_str(&format!("l{}:\n\t{}\n", label, value));
        }
    }
    x86_64.push_str(
        "\
        .text\n\
        .globl main\n\
        main:\n\
        \tpushq\t%rbp\n\
        \tmovq\t%rsp, %rbp\n\
        ",
    );

    x86_64.push_str(&body);

    x86_64.push_str(
        "\
        \tmovl\t$0, %eax\n\
        \tleave\n\
        \tret\n\n",
    );

    (x86_64, errors, warnings)
}
