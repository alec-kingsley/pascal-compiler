use crate::definitions::*;
use crate::tokenizer::*;

/// Reports a syntax error to the user if condition is true.
///
/// # Arguments
///
/// * `cond` - If true, report as error
/// * `code` - A string representing the user program.
/// * `i` - An index in the code from which the next token is erroneous.
/// * `err` - The error to report to the user.
///
/// # Examples
///
/// ```
/// let mut i = 0;
/// let code = "PROGRAM a BEGIN END;\n";
/// let mut token = next_token(code, &mut i);
/// token = next_token(code, &mut i);
/// syntax_check(last_token(code, &mut i) == ";", code, 10, "Expected ;");
/// ```
///
fn syntax_check(cond: bool, code: &str, mut i: usize, err: &str) {
    let length = next_token(code, &mut i).len();
    if !cond {
        report(code, i - length, i, err, "syntax");
    }
}

/// Parse program into ast.
///
/// "PROGRAM" ("(" identifier ("," identifer)* ")")? ";" block "."
///
/// # Arguments
/// * `code` - A string representing the user program.
/// 
pub fn parse_program(code: &str) -> Program {
    // return value
    let mut program = Program {
        body: Block { constants: Vec::new(), local_variables: Vec::new(), body: Statement::StatementList(Vec::new()) },
    };
    let mut i: usize = 0;
    
    // this variable holds a single token to view what it is
    let mut peeker = last_token(code, &mut i);
    syntax_check(peeker == "PROGRAM", code, i, "Missing PROGRAM keyword");
    next_token(code, &mut i);

    syntax_check(is_valid_identifier(&last_token(code, &mut i)), code, i, "Invalid identifier");
    next_token(code, &mut i); // This is the program keyword. Currently not used.
    

    peeker = last_token(code, &mut i);
    if peeker == "(" {
        while peeker != ")" {
            peeker = next_token(code, &mut i); // throw away file args for now
        }
    }

    peeker = last_token(code, &mut i);
    syntax_check(peeker == ";", 
            code, i, "Expected ;");
    next_token(code, &mut i);

    program.body = parse_block(code, &mut i);
   
    syntax_check(last_token(code, &mut i) == ".", 
            code, i, "Invalid program terminator");

    program
}


/// Parse block.
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
/// 
fn parse_block(code: &str, i: &mut usize) -> Block {

    let mut peeker = next_token(code, i);
    let mut constants = Vec::new();
	let mut local_variables = Vec::new();

    let mut seen_const = false;
    let mut seen_var = false;
	// search for body (only required part of a block)
    while peeker != "BEGIN" {
        // TODO - Parse labels, types, procedures, functions
        // parse constant block
        if peeker == "CONST" {
            syntax_check(!seen_const, code, *i, "Multiple CONST blocks encountered");
            seen_const = true;
            while is_valid_identifier(&last_token(code, i)) {
                let name = next_token(code, i);
                syntax_check(last_token(code, i) == "=", code, *i, "Expected =");
                next_token(code, i);
                let value = parse_expression(code, i);
                syntax_check(last_token(code, i) == ";", code, *i, "Expected ;");
                next_token(code, i);
                constants.push(Constant { name, value });
            }
        }

        // parse variable block
        if peeker == "VAR" {
            syntax_check(!seen_var, code, *i, "Multiple VAR blocks encountered");
            seen_var = true;
            while is_valid_identifier(&last_token(code, i)) {
                let mut identifiers = Vec::new();
                identifiers.push(next_token(code, i));
                while last_token(code, i) == "," {
                    next_token(code, i);
                    let identifier = last_token(code, i);
                    syntax_check(is_valid_identifier(&identifier), code, *i, "Invalid identifier");
                    next_token(code, i);
                    identifiers.push(identifier);
                }
                syntax_check(last_token(code, i) == ":", code, *i, "Expected : or ,");
                next_token(code, i);
                let tipe = parse_type(code, i);
                syntax_check(last_token(code, i) == ";", code, *i, "Expected ;");
                next_token(code, i);
                for identifier in identifiers {
                    local_variables.push(Variable { name: identifier, tipe: tipe.clone() });
                }
            }
        }
        peeker = next_token(code, i);
    }
    
	Block { constants, local_variables, body: parse_statement_list(code, i) }
}

/// Parse type.
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_type(code: &str, i: &mut usize) -> SuperType {
    let peeker = next_token(code, i);
    match peeker.as_str() {
        "INTEGER" => SuperType::Integer,
        "BOOLEAN" => SuperType::Boolean,
        "REAL" => SuperType::Real,
        "CHAR" => SuperType::Char,
        "STRING" => SuperType::Stryng,
        "TEXT" => SuperType::Text,
        "PACKED" => {
            // For now I'm not worried about implementing packed, but functionally
            // it's about the same so I'll leave this
            syntax_check(last_token(code, i) == "ARRAY", code, *i, "Expected ARRAY");
            parse_type(code, i)
        },
        "ARRAY" => {
            syntax_check(last_token(code, i) == "[", code, *i, "Expected [");
            next_token(code, i);

            let start_idx = parse_expression(code, i);
            syntax_check(last_token(code, i) == "..", code, *i, "Expected ..");
            next_token(code, i);

            let end_idx = parse_expression(code, i);
            syntax_check(last_token(code, i) == "]", code, *i, "Expected ]");
            next_token(code, i);
            syntax_check(last_token(code, i) == "OF", code, *i, "Expected OF");
            next_token(code, i);
            let tipe = parse_type(code, i);
            SuperType::Array(Box::new(tipe), start_idx, end_idx)
        },
        _ => { report(code, *i - peeker.len(), *i, "Failed to parse type", "syntax"); panic!() }
    }
}

/// Parse statement.
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_statement(code: &str, i: &mut usize) -> Statement {
    let peeker = next_token(code, i);
    if peeker == "BEGIN" {
        parse_statement_list(code, i)
    } else if peeker == "IF" {
        parse_if_statement(code, i)
    } else if peeker == "WHILE" {
        parse_while_loop(code, i)
    } else if peeker == "REPEAT" {
        parse_repeat_loop(code, i)
    } else if peeker == "FOR" {
        parse_for_loop(code, i)
    } else if is_valid_identifier(&peeker) {
        if last_token(code, i) == ":=" {
            let start = *i - peeker.len();
            next_token(code, i);
            let expression = parse_expression(code, i);
            let end = *i;
            Statement::Assignment(peeker, expression, start, end)
        } else if last_token(code, i) == "[" {
            let start = *i - peeker.len();
            next_token(code, i);
            let index = parse_expression(code, i);
            syntax_check(last_token(code, i) == "]", code, *i, "Expected ]");
            next_token(code, i);
            syntax_check(last_token(code, i) == ":=", code, *i, "Expected :=");
            next_token(code, i);
            let expression = parse_expression(code, i);
            let end = *i;
            Statement::ElementAssignment(peeker, index, expression, start, end)

        } else {
            // TODO - There's a difference between these, handle it
            if peeker == "READ" || peeker == "READLN" {
                *i -= peeker.len();
                parse_read_call(code, i)   
            } else {
                parse_procedure_call(&peeker, code, i)
            }
        }
    } else {
        report(code, *i - peeker.len(), *i, "Unrecognized statement", "syntax");
        panic!()
    }

}

/// Parse statement list.
///
/// A statement list is a list of statements surrounded by "BEGIN" and "END"
/// Prereq: "BEGIN" already consumed
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_statement_list(code: &str, i: &mut usize) -> Statement {
    // return value
    let mut statement_list = Vec::new();

    let mut peeker = last_token(code, i);
    syntax_check(peeker != "END", code, *i, "Empty statement list");
    while peeker != "END" {
        statement_list.push(parse_statement(code, i));
        peeker = last_token(code, i);
        syntax_check(peeker == ";" || peeker == "END", code, *i, "Expected ; or END");
        peeker = next_token(code, i);
    }
    
   	Statement::StatementList(statement_list)
}

/// Parse if statement.
///
/// "IF" expression "THEN" statement ("ELSE" statement)?
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_if_statement(code: &str, i: &mut usize) -> Statement {
    let condition_start = *i;
    let condition = parse_expression(code, i);
    let condition_end = *i;
    let peeker = last_token(code, i);
    syntax_check(peeker.as_str() == "THEN", 
            code, *i, "Missing THEN after IF");
    next_token(code, i);

    let true_body = parse_statement(code, i);

    // check if it has an ELSE clause
    let false_body = if last_token(code, i) == "ELSE" {
        next_token(code, i);
        parse_statement(code, i)
    } else {
        Statement::DoNothing
    };
    Statement::IfStatement(condition, Box::new(true_body), Box::new(false_body), condition_start, condition_end)
}

/// Parse while loop.
///
/// "WHILE" condition "DO" statement
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_while_loop(code: &str, i: &mut usize) -> Statement {
    // parse condition
    let condition_start = *i;
    let condition = parse_expression(code, i);
    let condition_end = *i;
    let peeker = last_token(code, i);

    syntax_check(peeker.as_str() == "DO", code, *i, "Missing DO after WHILE");
    
    next_token(code, i);
    let body = parse_statement(code, i);
    Statement::WhileLoop(condition, Box::new(body), condition_start, condition_end)
}

/// Parse repeat loop.
///
/// "REPEAT" statement (";" statement)* "UNTIL" condition
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_repeat_loop(code: &str, i: &mut usize) -> Statement {
    // return value
    let mut statement_list = Vec::new();

    let mut peeker = last_token(code, i);
    syntax_check(peeker != "UNTIL", code, *i, "Empty statement list");
    while peeker != "UNTIL" {
        statement_list.push(parse_statement(code, i));
        peeker = last_token(code, i);
        syntax_check(peeker == ";" || peeker == "UNTIL", code, *i, "Expected ; or UNTIL");
        peeker = next_token(code, i);
    }
    
    let condition_start = *i;
    let condition = parse_expression(code, i);
    let condition_end = *i;
   	Statement::RepeatLoop(condition, Box::new(Statement::StatementList(statement_list)), condition_start, condition_end)

}



/// Parse for loop.
///
/// "FOR" identifier ":=" expression ("TO"|"DOWNTO") expression "DO" statement
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_for_loop(code: &str, i: &mut usize) -> Statement {
    
    let name_start = *i;

    // ensure valid identifier, save
    let mut peeker = last_token(code, i);
    syntax_check(is_valid_identifier(&peeker), code, *i, "Invalid identifier");
    next_token(code, i);
    let identifier = peeker;
    let name_end = *i;

    // ensure followed by :=
    syntax_check(last_token(code, i) == ":=", code, *i, "Expected :=");
    next_token(code, i);
    
    // get range
    let range_start = *i;
    let start = parse_expression(code, i);
    peeker = last_token(code, i);
    syntax_check(peeker == "TO" || peeker == "DOWNTO", code, *i, "Expected TO or DOWNTO");
    next_token(code, i);
    let ascending = peeker == "TO";
    let end = parse_expression(code, i);
    let range_end = *i;

    // ensure followed by DO
    syntax_check(last_token(code, i) == "DO", code, *i, "Expected DO");
    next_token(code, i);

    // get body, return
    let body = parse_statement(code, i);
    Statement::ForLoop(identifier, name_start, name_end, start, end, range_start, range_end, ascending, Box::new(body))
}

/// Parse procedure call.
///
/// identifier ("(" expression ("," expression)* ")")?
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_procedure_call(procedure_identifier: &str, code: &str, i: &mut usize) -> Statement {
    let start = *i - procedure_identifier.len();

    syntax_check(is_valid_identifier(procedure_identifier), 
            code, *i - procedure_identifier.len(), "Invalid identifier");
    
    let mut arguments: Vec<Expression> = Vec::new();
    let mut peeker = next_token(code, i);
    if peeker != "(" {
        // a procedure can be called with no inputs, in which case we shouldn't've taken anything
        // off the stack.
        *i -= peeker.len();
    } else {
        while peeker != ")" {
            // could be a procedure identifier as well, but a single identifier is a valid
            // expression.
            arguments.push(parse_expression(code, i));
            
            // remove next comma or )
            peeker = next_token(code, i);
        }
    }
    let end = *i;
    Statement::ProcedureCall(procedure_identifier.to_string(), arguments, start, end)

}

/// Parse read call.
/// This is special since unlike other procedures, it accepts specifically variable names.
///
/// "READ" "(" identifier ("," identifier)* ")"
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_read_call(code: &str, i: &mut usize) -> Statement {
    let start = *i;
    next_token(code, i);
    let mut variable_list = Vec::new();
    syntax_check(last_token(code, i) == "(", code, *i, "Expected (");
    let mut peeker = next_token(code, i);
    while peeker != ")" {
        syntax_check(is_valid_identifier(&last_token(code, i)), code, *i, "Invalid identifier");
        variable_list.push(next_token(code, i));
        // consume comma or )
        peeker = next_token(code, i);
    }
    let end = *i;
    Statement::ReadCall(variable_list, start, end)
}

/// Parse expression.
///
/// Either returns a simple expression or a boolean operation involving them.
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_expression(code: &str, i: &mut usize) -> Expression {
    let start = *i;

    let operand1 = parse_simple_expression(code, i);
    
    // operator and operand2 are optional
    let operator = if is_equality_operator(&last_token(code, i)) {
        next_token(code, i)
    } else {
        "NONE".to_string()
    };
    let operand2 = if operator == "NONE" {
        operand1.clone()
    } else {
        parse_simple_expression(code, i)
    };
    
    let end = *i;

    // create and return expression
    Expression { start, end, operand1, operand2, operator, }

}

/// Parse simple expression.
///
/// +, -, OR
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_simple_expression(code: &str, i: &mut usize) -> SimpleExpression {
    let start = *i; 
    // take off + or - if present, set positive appropriately
    let positive = match last_token(code, i).as_str() {
        "+" => {next_token(code, i); true},
        "-" => {next_token(code, i); false},
        _   => true,
    };
    let mut operators: Vec<String> = Vec::new();
    let mut operands: Vec<Term> = Vec::new();
    operands.push(parse_term(code, i));

    while matches!(last_token(code, i).as_str(), "+" | "-" | "OR") {
        operators.push(next_token(code, i));
        operands.push(parse_term(code, i));
    } 
     
    
    let end = *i;
    SimpleExpression { start, end, positive, operands, operators, }

}


/// Parse term.
///
/// * / DIV MOD AND 
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_term(code: &str, i: &mut usize) -> Term {
    let start = *i;

    let mut operators: Vec<String> = Vec::new();
    let mut operands: Vec<Factor> = Vec::new();
    // must be at least one factor
    operands.push(parse_factor(code, i));
    while matches!(last_token(code, i).as_str(), "*" | "/" | "DIV" | "MOD" | "AND") {
        operators.push(next_token(code, i));
        operands.push(parse_factor(code, i));
    }
   
    let end = *i;
    Term { start, end, operands, operators, }
}

/// Parse factor.
///
/// Literals, NOT, parentheticals, identifiers, constants
///
/// # Arguments
/// * `code` - A string representing the user program.
/// * `i` - An index within `code` representing a point after the last token processed.
///
fn parse_factor(code: &str, i: &mut usize) -> Factor {

    let start = *i;
    let mut peeker = next_token(code, i);

    if is_valid_identifier(&peeker) {
        let identifier = peeker.clone();
        if last_token(code, i) == "[" {
            next_token(code, i);
            let index = parse_expression(code, i);
            syntax_check(last_token(code, i) == "]", code, *i, "Expected ]");
            next_token(code, i);
            let end = *i;
            Factor::ArrayIndex(identifier, index, start, end)
        } else {
            let mut arguments: Vec<Expression> = Vec::new();
            if last_token(code, i) == "(" {
                next_token(code, i);
                while peeker != ")" {
                    arguments.push(parse_expression(code, i));
                    peeker = next_token(code, i);
			    }
		    }
            let end = *i;
		    Factor::Identifier(identifier, arguments, start, end)
        }
    // expression in parentheses
    } else if peeker == "(" {
        let factor = Factor::Parenthetical(parse_expression(code, i));
        peeker = last_token(code, i);
        syntax_check(peeker == ")", 
                code, *i, "Unclosed (");
        next_token(code, i);
        factor

    // negated factor
    } else if peeker == "NOT" {
        let factor = parse_factor(code, i);
        let end = *i;
        Factor::NegatedFactor(Box::new(factor), start, end)
        
    // list of expressions / ranges
    } else if peeker == "[" {
        let mut expression_list: Vec<ExpressionOrRange> = Vec::new();
        while peeker != "]" {
            let expression1 = parse_expression(code, i);
            peeker = next_token(code, i);
            expression_list.push(
                if peeker.as_str() == ".." {
                    let pusher = ExpressionOrRange::Range(expression1, parse_expression(code, i));
                    next_token(code, i);
                    pusher
                } else {
                    ExpressionOrRange::Expression(expression1)
                }
            );

        }
        Factor::List(expression_list)
    
    // constant
    } else {
        if peeker == "NIL" {
            Factor::Constant(UnsignedConstant::Nil(*i - "NIL".len()))
        } else {
            match peeker.parse::<u64>() {
                Ok(n) => { 
                    if last_token(code, i) == "." {
                        next_token(code, i);
                        let decimal = match last_token(code, i).parse::<u64>()  {
                            Ok(f) => {next_token(code, i); f},
                            Err(_) => {
                                peeker = next_token(code, i); 
                                report(code, *i - peeker.len(), *i, "Expected number","syntax");
                                panic!()
                            },
                        };
                        let mut power = 1;
                        let mut temp_decimal = decimal;
                        while temp_decimal > 0 {
                            temp_decimal /= 10;
                            power *= 10;
                        }
                        Factor::Constant(UnsignedConstant::UnsignedReal(n as f64 + decimal as f64 / power as f64))
                    } else {
                        Factor::Constant(UnsignedConstant::UnsignedInteger(n))
                    }
                },
                Err(_) => {
                        syntax_check(peeker.starts_with('\''), code, *i - peeker.len(), "Failed to parse factor");
                        let quote = &peeker[1..peeker.len()-1]; // remove the ''
                        if quote.len() == 1 {
                            let ch = quote.chars().next().unwrap();
                            if ch.is_ascii() {
                                Factor::Constant(UnsignedConstant::Char(ch as u8))
                            } else {
                                Factor::Constant(UnsignedConstant::Quote(quote.to_string()))
                            }
                        } else {
                            Factor::Constant(UnsignedConstant::Quote(quote.to_string()))
                        }
                },
            }
        }

    }

}

/// Returns "true" iff `token` is an equality operator, given by the equality_operators vector.
///
/// # Arguments
/// * `token` - A string representing the token to test.
///
fn is_equality_operator(token: &str) -> bool {
    // set up equality operators
    let equality_operators = vec!["<", "<=", "=", "<>", ">=", ">", "IN"];
    equality_operators.contains(&token)
}

/// Returns "true" iff `token` is a valid identifier, and not reserved.
///
/// # Arguments
/// * `token` - A string representing the token to test.
///
fn is_valid_identifier(token: &str) -> bool {
    let mut is_valid = true;
    
    // ensure that first character is a letter A-Z
    if !token.chars().next().unwrap().is_ascii_alphabetic() {
        is_valid = false;
    } else {
        for c in token.chars().skip(1) {
            if !c.is_alphanumeric() {
                is_valid = false;
            }
        }
    }

    // set up reserved words
    let reserved_words = ["AND", "ARRAY", "BEGIN", "CASE", "CONST", 
            "DIV", "DO", "DOWNTO", "ELSE", "END", "FILE", "FOR", 
            "FUNCTION", "GOTO", "IF", "IN", "LABEL", "MOD", "NIL",
            "NOT", "OF", "OR", "PACKED", "PROCEDURE", "PROGRAM",
            "RECORD", "REPEAT", "SET", "THEN", "TO", "TYPE", "UNTIL",
            "VAR", "WHILE", "WITH"];

    is_valid && !reserved_words.contains(&token)
}


