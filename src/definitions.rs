use std::process;

/// Reports an error to the user.
///
/// # Arguments
///
/// * `code` - A string representing the user program.
/// * `start` - Start of erroneous section of code. (first char)
/// * `end` - End of erroneous section of code. (after last char)
/// * `err` - The error to report to the user.
/// * `variant` - "syntax" (in which case panic), "error", or "warning"
///
/// # Examples
///
/// ```
/// let code = "PROGRAM a BEGIN END;\n";
/// report(code, 10, 15, "Expected ;", "syntax");
///
/// # Output:
///
/// Syntax error at line 1, character 10.
/// Expected ;
/// PROGRAM a BEGIN END
///           ^^^^^
///
/// ```
///
pub fn report(code: &str, start: usize, end: usize, err: &str, variant: &str) {
    assert!(end <= code.len(), "Invalid code index.");
    assert!(start < end, "`start` must be less than `end`");

    let mut line_idx = 0;
    let mut char_idx = 0;
    let mut chars_left = start;

    // find the line and char index of start
    for (_, c) in code.char_indices() {
        if chars_left == 0 {
            break;
        }
        if c == '\n' {
            line_idx += 1;
            char_idx = 0;
        } else {
            char_idx += 1;
        }
        chars_left -= 1;
    }
    // find index of end of the line after end
    let mut end_of_line_idx: usize = end;
    while end_of_line_idx < code.len() && code.chars().nth(end_of_line_idx).unwrap() != '\n' {
        end_of_line_idx += 1;
    }

    let mut block = String::from(&code[start - char_idx..end_of_line_idx]);
    
    block.insert_str(char_idx + end - start, "\x1b[38;5;208m"); // return to orange after error
    block.insert_str(char_idx, "\x1b[31m"); // switch to red for error
    
    match variant {
        "syntax" => print!("\x1b[31m\nSyntax error \x1b[0m"), // red
        "error" => print!("\x1b[31m\nError \x1b[0m"), // red
        "varning" => print!("\x1b[33m\nWarning \x1b[0m"), // yellow
        _   => panic!("Unknown error"),
    }

    println!("{}", &format!("\
            at line {}, character {}:\n\
            {}\n\
            \x1b[38;5;208m{}\x1b[0m", line_idx + 1, char_idx + 1, err, block));

    if variant == "syntax" {
        process::exit(1);
    }

}

// these definitions are from Peter Grogono's Programming in Pascal (1978)

pub struct Program {
    pub body: Block,
}
pub struct Block {
    pub constants: Vec<Constant>,
    pub local_variables: Vec<Variable>,
    pub body: Statement,
}

pub struct Constant {
    pub name: String,
    pub value: Expression,
}

pub struct Variable {
    pub name: String,
    pub tipe: SuperType, // misspelt since Rust doesn't allow "type"
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    Real,
    Char,
    Stryng, // misspelt since Rust doesn't allow "String"
    Text,
    // (type, start index, end index) (why tf does pascal allow this)
    // TODO - Pascal arrays are way more complex. This is not sufficient.
    Array(Box<Type>, isize, isize),
    Undefined,
}

// for AST, Compiler changes to Type
#[derive(Clone)]
pub enum SuperType {
    Integer,
    Boolean,
    Real,
    Char,
    Stryng,
    Text,
    Array(Box<SuperType>, Expression, Expression),
}

#[derive(Clone)]
pub enum Statement {
    DoNothing, // this is for empty blocks

    // could be for a variable or the function name (return value)
    // (variable, expression, start, end)
    Assignment(String, Expression, usize, usize),
    // this is for array indexing
    // (variable, index expression, expression, start, end
    ElementAssignment(String, Expression, Expression, usize, usize),
    // (identifier, arguments, start, end)
    ProcedureCall(String, Vec<Expression>, usize, usize),
    // (variables, start, end)
    ReadCall(Vec<String>, usize, usize), // this is its own thing since it can't take expressions...
    // (condition, case:true, case:false, condition_start, condition_end)
    IfStatement(Expression, Box<Statement>, Box<Statement>, usize, usize),
    // (condition, body, condition_start, condition_end)
    WhileLoop(Expression, Box<Statement>, usize, usize),
    // (condition, body, condition_start, condition_end)
    RepeatLoop(Expression, Box<Statement>, usize, usize),
    // (identifier, identifier_start, identifier_end, starting expression, ending expression, range_start, range_end, ascending?, statement)
    ForLoop(String, usize, usize, Expression, Expression, usize, usize, bool, Box<Statement>),
    // StatementList must be of form:
    // BEGIN
    //   statement;
    //   statement;
    //   ...
    // END
    StatementList(Vec<Statement>),
}

#[derive(Clone)]
pub struct Expression {
    pub start: usize,
    pub end: usize,
    pub operand1: SimpleExpression,
    pub operand2: SimpleExpression,
    pub operator: String,
}
#[derive(Clone)]
pub struct SimpleExpression {
    pub start: usize,
    pub end: usize,
    pub positive: bool, // only applies to first term
    pub operands: Vec<Term>,
    pub operators: Vec<String>,
}
#[derive(Clone)]
pub struct Term {
    pub start: usize,
    pub end: usize,
    pub operands: Vec<Factor>,
    pub operators: Vec<String>,
}
#[derive(Clone)]
pub enum Factor {
    Constant(UnsignedConstant),
    // this could be a function call or variable name
    // (identifier, arguments, start, end)
    Identifier(String, Vec<Expression>, usize, usize),
    ArrayIndex(String, Expression, usize, usize),
    Parenthetical(Expression),
    // (factor, start, end)
    NegatedFactor(Box<Factor>, usize, usize),
    List(Vec<ExpressionOrRange>)
}
#[derive(Clone)]
pub enum ExpressionOrRange {
    Expression(Expression),
    Range(Expression, Expression),
}
#[derive(Clone)]
pub enum UnsignedConstant {
    // could also have an identifier for a constant
    UnsignedInteger(u64),
    UnsignedReal(f64),
    // (starting location)
    Nil(usize),
    Quote(String),
    Char(u8),
}



