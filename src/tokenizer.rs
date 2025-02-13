use std::collections::HashSet;
use std::process;

// get symbol at location i, if non-existent return ""
fn get_symbol(code: &str, i: usize) -> String {
    // pascal punctuation
    // NOTE: Some say that AND/OR/NOT are punctuation, however I decided to tokenize them as
    // identifiers instead. So for example, "sand" and "more" are valid identifiers.
    let symbols: HashSet<&str> = vec![
        "+", "-", "*", "/", "<", "≤", "<=", "=", "≠", "<>", "≥", ">=", ">", "∧", "∨", "¬", "~",
        ":=", ",", ";", ",", ";", ":", "'", ".", "..", "(", ")", "[", "(.", "]", ".)", "//", "{",
        "(*", "}", "*)",
    ]
    .into_iter()
    .collect();

    // Comment symbols:
    // // or {} or (**)
    // these should be removed by tokenizer

    // token to be returned
    let mut token = String::new();
    for symbol in symbols.iter() {
        if symbol.len() > token.len()
            && i + symbol.len() <= code.len()
            && *symbol == &code[i..i + symbol.len()]
        {
            token = symbol.to_string();
        }
    }

    token
}

// return token at location, if comment try again
// prereq: i < |code|
// comments should be handled recursively, so {{}} is acceptable, but {*) is not
fn next_token_runner(code: &str, i: &mut usize, ignore_special: bool) -> String {
    while *i < code.len() && code[*i..].chars().next().unwrap().is_whitespace() {
        *i += 1; // Skip whitespace
    }

    let mut token: String = get_symbol(code, *i);
    *i += token.len();

    match token.as_str() {
        // handle block comment type 1
        "{" => {
            while *i < code.len() && next_token_runner(code, i, true) != "}" {}
            token.clear();
        }
        // handle block comment type 2
        "(*" => {
            while *i < code.len() && next_token_runner(code, i, true) != "*)" {}
            token.clear();
        }

        // handle inline comment
        "//" => {
            while *i < code.len() && &code[*i..*i + 1] != "\n" {
                *i += 1;
            }
            *i += 1; // Skip the newline
            token.clear();
        }

        // quotes should preserve whitespace, as such should be their own symbol
        "'" if !ignore_special => {
            while *i < code.len() {
                let ch = code[*i..].chars().next().unwrap();
                token.push(ch);
                *i += ch.len_utf8();
                if ch == '\'' {
                    break;
                }
            }
            assert!(
                code[*i - 1..].chars().next().unwrap() == '\'',
                "Unmatched ' found"
            )
        }

        // pascal synonyms
        "≤" => {
            token = "<=".to_string();
        }
        "≠" => {
            token = "<>".to_string();
        }
        "≥" => {
            token = ">=".to_string();
        }
        "∧" => {
            token = "AND".to_string();
        }
        "∨" => {
            token = "OR".to_string();
        }
        "¬" | "~" => {
            token = "NOT".to_string();
        }
        "(." => {
            token = "[".to_string();
        }
        ".)" => {
            token = "]".to_string();
        }

        "" => {
            while *i < code.len() {
                let ch = code[*i..].chars().next().unwrap();
                if ch.is_whitespace() || !get_symbol(code, *i).is_empty() {
                    break;
                }
                token.push(ch);
                *i += ch.len_utf8();
            }

            // pascal is not case-sensitive, so we make all identifiers uppercase
            token = token.to_uppercase();
        }
        _ => {}
    }

    if token.len() > 0 {
        token
    } else if *i < code.len() {
        next_token_runner(code, i, false)
    } else {
        "".to_string()
    }
}

// get next token from source code at index i
pub fn next_token(code: &str, i: &mut usize) -> String {
    let result = next_token_runner(code, i, false);

    if result.is_empty() {
        println!("\x1b[31mUnexpected end of input\x1b[0m");
        process::exit(1)
    } else {
        result
    }
}

// get next token without updating i
pub fn last_token(code: &str, i: &mut usize) -> String {
    let mut j = *i;
    next_token(code, &mut j)
}
