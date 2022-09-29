#![allow(unused)]
use std::{fs::File, io::Read};

/*
H1
Text('This is my heading')
HorizontalRule
Code
Text('int main() {')
Text('}')
Code
*/

/*
# <text> \n
<h1> <text> <\h1>\n

## <text> \n
<h2> <text> <\h2>\n

*<text>*
<i><text></i>
*/

#[derive(Debug)]
enum Token {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    HorizontalRule,
    ///`*<text>*`
    Italic,
    ///`**<text>**`
    Bold,
    ///`<text>`
    InlineCode,
    Code,
    ///Stores the level of indentation.
    BlockQuote(usize),
    ExclamationMark,
    OpenBracket,
    CloseBracket,
    OpenParentheses,
    CloseParentheses,
    NewLine,
    Char(char),
    Number(u8),
    FullStop,
    Tab,
}

//This system is not robust enough to do italics **Unspecified amount of text**.
fn main() {
    let mut file = File::open("example.md").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();

    let mut tokens: Vec<Token> = Vec::new();
    let mut start = true;

    let mut iter = string.chars().peekable();
    while let Some(char) = iter.next() {
        match char {
            '#' if start => {
                let mut h = 1;
                loop {
                    if iter.peek() == Some(&'#') {
                        iter.next();
                        h += 1;
                    } else {
                        break;
                    }
                }

                match h {
                    1 => tokens.push(Token::H1),
                    2 => tokens.push(Token::H2),
                    3 => tokens.push(Token::H3),
                    4 => tokens.push(Token::H4),
                    5 => tokens.push(Token::H5),
                    6 => tokens.push(Token::H6),
                    _ => unreachable!(),
                }
            }
            '-' if start => {
                if iter.peek() == Some(&'-') {
                    iter.next();
                    if iter.peek() == Some(&'-') {
                        iter.next();
                        tokens.push(Token::HorizontalRule);
                    }
                }
            }
            '>' if start => {
                let mut level = 1;
                loop {
                    match iter.peek() {
                        Some(&'>') => {
                            iter.next();
                            level += 1;
                        }
                        Some(&' ') => {
                            iter.next();
                        }
                        _ => break,
                    }
                }
                tokens.push(Token::BlockQuote(level));
            }
            '`' if start => {
                if iter.peek() == Some(&'`') {
                    iter.next();

                    if iter.peek() == Some(&'`') {
                        iter.next();

                        tokens.push(Token::Code);
                    }
                }
            }
            '`' => tokens.push(Token::InlineCode),
            '*' => {
                if iter.peek() == Some(&'*') {
                    iter.next();
                    tokens.push(Token::Bold);
                } else {
                    tokens.push(Token::Italic);
                }
            }
            '\n' => {
                start = true;
                tokens.push(Token::NewLine);
            }
            '\t' => tokens.push(Token::Tab),
            '!' => tokens.push(Token::ExclamationMark),
            '[' => tokens.push(Token::OpenBracket),
            ']' => tokens.push(Token::CloseBracket),
            '(' => tokens.push(Token::OpenParentheses),
            ')' => tokens.push(Token::CloseParentheses),
            '0'..='9' => tokens.push(Token::Number(char as u8 - 48)),
            '.' => tokens.push(Token::FullStop),
            _ => {
                // tokens.push(Token::Char(char));
            }
        }
        if char != '\n' && start {
            start = false;
        }
    }
    dbg!(tokens);
}
