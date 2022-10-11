#![allow(unused)]
use std::{fs::File, io::Read, iter::Peekable, slice::Iter};

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

/*
Tables:

This is probably the hardest thing to convert.

<table>
  <tr>
    <td>Emil</td>
    <td>Tobias</td>
    <td>Linus</td>
  </tr>
  <tr>
    <td>16</td>
    <td>14</td>
    <td>10</td>
  </tr>
</table>

| Emil | Tobias | Linus |
| ---- | ------ | ----- |
|  16  |   14   |   10  |
 */

#[derive(Debug)]
enum Token {
    Hash,
    Italic,
    Bold,
    BackTick,
    BlockQuote,
    ExclamationMark,
    OpenBracket,
    CloseBracket,
    OpenParentheses,
    CloseParentheses,
    NewLine,
    CarriageReturn,
    Pipe,
    Hyphen,
    String(String),
    Tab,
    Hiphen,
}

///Some of the basic conversions
// fn convert(token: Token) -> &'static str {
//     match token {
//         Token::H1 => "<h1>",
//         Token::H2 => "<h2>",
//         Token::H3 => "<h3>",
//         Token::H4 => "<h4>",
//         Token::H5 => "<h5>",
//         Token::H6 => "<h6>",
//         Token::HorizontalRule => "<hr>",
//         Token::Italic => "<i>",
//         Token::Bold => "<b>",
//         Token::InlineCode => "<code>",
//         Token::Code => "<code>",
//         Token::BlockQuote => "<blockquote>",
//         Token::NewLine => "<br>",
//         Token::Tab => "&emsp",
//         _ => todo!(),
//     }
// }

//This system is not robust enough to do italics **Unspecified amount of text**.
//https://alajmovic.com/posts/writing-a-markdown-ish-parser/

//This markdown parser won't support proper markdown.
//No __text__ or _text_ or maybe no **text** or *text*.
//No links with titles [test](http://link.net "title")
//No * or ~ for lists.
fn main() {
    let mut file = File::open("test.md").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();

    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = string.chars().peekable();

    let mut string = String::new();
    let mut string_changed = false;

    //TODO: I am doing the lexing and parsing in the same step.
    //This is already to difficult to manage.
    //Remove the H1-H6 + All of the iter.peek related logic.
    //All of the start stuff is dumb too.
    while let Some(char) = iter.next() {
        match char {
            '#' => tokens.push(Token::Hash),
            '-' => tokens.push(Token::Hiphen),
            '-' => tokens.push(Token::Hyphen),
            '>' => tokens.push(Token::BlockQuote),
            '`' => tokens.push(Token::BackTick),
            '*' => {
                if iter.peek() == Some(&'*') {
                    iter.next();
                    tokens.push(Token::Bold);
                } else {
                    tokens.push(Token::Italic);
                }
            }
            '\n' => tokens.push(Token::NewLine),
            '\r' => tokens.push(Token::CarriageReturn),
            '\t' => tokens.push(Token::Tab),
            '!' => tokens.push(Token::ExclamationMark),
            '[' => {
                tokens.push(Token::OpenBracket);
            }
            ']' => tokens.push(Token::CloseBracket),
            '(' => tokens.push(Token::OpenParentheses),
            ')' => tokens.push(Token::CloseParentheses),
            '|' => tokens.push(Token::Pipe),
            ' ' if !string.is_empty() => {
                string.push(char);
                string_changed = true;
            }
            ' ' => (),
            _ => {
                string.push(char);
                string_changed = true;
            }
        }

        if string_changed {
            string_changed = false;
        } else if !string.is_empty() {
            tokens.insert(tokens.len() - 1, Token::String(string));
            string = String::new();
        }
    }
    dbg!(&tokens);
    parse(&tokens);
}

#[derive(Debug)]
enum Expr {
    ///Level, Text
    Heading(u8, String),
    Bold(String),
    Italic(String),
    ///Number, Content
    NumberedList(u32, String),
    List(Vec<Expr>),
    ///Title, Reference/Link
    Link(String, String),
    Text(String),
}

fn parse(tokens: &[Token]) {
    let mut iter = tokens.iter().peekable();

    let ast: Vec<Expr> = Vec::new();

    while let Some(token) = iter.next() {
        if let Some(expr) = expression(token, &mut iter) {
            dbg!(expr);
        }
    }
}

fn expression(token: &Token, iter: &mut Peekable<Iter<Token>>) -> Option<Expr> {
    match token {
        //How to do better?
        Token::OpenBracket => {
            //Full
            if let Some(Token::String(title)) = iter.peek() {
                iter.next();
                if let Some(Token::CloseBracket) = iter.peek() {
                    iter.next();
                    if let Some(Token::OpenParentheses) = iter.peek() {
                        iter.next();
                        if let Some(Token::String(link)) = iter.peek() {
                            iter.next();
                            if let Some(Token::CloseParentheses) = iter.peek() {
                                iter.next();
                                return Some(Expr::Link(title.clone(), link.clone()));
                            }
                        }
                    }
                }
            }

            //Empty
            if let Some(Token::CloseBracket) = iter.peek() {
                iter.next();
                if let Some(Token::OpenParentheses) = iter.peek() {
                    iter.next();
                    if let Some(Token::CloseParentheses) = iter.peek() {
                        iter.next();
                        return Some(Expr::Link(String::new(), String::new()));
                    }
                }
            }

            return None;
        }
        Token::Hash => {
            let mut level = 1;
            loop {
                match iter.next() {
                    Some(Token::Hash) => {
                        if level > 6 {
                            unreachable!()
                        }
                        level += 1;
                    }
                    Some(Token::String(string)) => {
                        return Some(Expr::Heading(level, string.clone()));
                    }
                    _ => unreachable!(),
                }
            }
        }
        //--- HorizontalRule
        //|----| Table
        Token::Hiphen => {}
        Token::String(string) => {
            let mut chars = string.chars();
            let mut number = String::new();

            //Match lists:
            //1. First item
            //2. Second item
            //Currently the order doesn't matter and lists are not grouped.
            //Weird orders like 3. 2. 1. are technically valid.
            while let Some(char) = chars.next() {
                if char.is_numeric() {
                    number.push(char);
                } else if char == '.' && !number.is_empty() && chars.next() == Some(' ') {
                    if let Ok(number) = number.parse() {
                        return Some(Expr::NumberedList(number, chars.collect()));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            return Some(Expr::Text(string.clone()));
        }
        // **This is some bold text**
        Token::Bold => {
            if let Some(Token::String(string)) = iter.peek() {
                iter.next();
                return Some(Expr::Bold(string.clone()));
            }
        }
        // *This is some italic text*
        Token::Italic => {
            if let Some(Token::String(string)) = iter.peek() {
                iter.next();
                return Some(Expr::Italic(string.clone()));
            }
        }
        _ => (),
    }
    None
}
