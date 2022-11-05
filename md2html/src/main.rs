#![feature(iter_advance_by)]
#![allow(unused)]
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    iter::Peekable,
    os::windows::prelude::MetadataExt,
    path::{Path, PathBuf},
    slice::Iter,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq)]
enum Token {
    ///Level
    Heading(u8),
    Italic,
    Bold,
    Strikethrough,
    BackTick,
    CodeBlock,
    BlockQuote,
    OpenBracket,
    CloseBracket,
    OpenParentheses,
    CloseParentheses,
    NewLine,
    CarriageReturn,
    String(String),
    Tab,
    TemplateStart,
    TemplateEnd,
    CommentStart,
    CommentEnd,
    Tag,
    Equal,
    ShortcodeStart,
    ShortcodeEnd,
    Plus,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Heading(level) => unreachable!(),
            Token::Plus => String::from("+++"),
            Token::Italic => String::from("*"),
            Token::Bold => String::from("**"),
            Token::Strikethrough => String::from("~~"),
            Token::BackTick => String::from("`"),
            Token::CodeBlock => String::from("```"),
            Token::BlockQuote => String::from(">"),
            Token::OpenBracket => String::from("["),
            Token::CloseBracket => String::from("]"),
            Token::OpenParentheses => String::from("("),
            Token::CloseParentheses => String::from(")"),
            Token::NewLine => String::from("\n"),
            Token::CarriageReturn => String::from("\r"),
            Token::String(string) => string.clone(),
            Token::Tab => String::from("\t"),
            Token::TemplateStart => String::from("{{"),
            Token::TemplateEnd => String::from("}}"),
            Token::CommentStart => String::from("<!-- "),
            Token::CommentEnd => String::from("-->"),
            Token::Tag => String::from("+++"),
            Token::Equal => String::from("="),
            Token::ShortcodeStart => String::from("{%"),
            Token::ShortcodeEnd => String::from("%}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Expr {
    ///Level, Text
    Heading(u8, String),
    ///Level, Text
    BlockQuote(u8, String),
    Bold(String),
    Italic(String),
    OrderedList(Vec<String>),
    List(Vec<Expr>),
    ///Title, Reference/Link
    Link(String, String),
    Text(String),
    Strikethrough(String),
    CodeBlock(Vec<String>),
    Code(String),
    HorizontalRule,
    Template(String),
    Variable(String),
}

/*
Posts are markdown files.
The posts page will compile a list of all the posts and using a post template.

for each post in "/posts"
get the post title, user, date, word count, read duration, file path (to use as hyperlink).

Then create the posts page using the posts template:

<body>
    <!-- {{ header }} -->
    <h1 style="text-align: center;">Posts</h1>
    <main class="list">
        <ul>
            <!-- {{ list_items }} -->
        </ul>
    </main>
</body>

I though I would need to inject code into my markdown files but that was wrong.
Altough that might be useful in the future for things like syntax highlighting/LaTeX.
*/

#[derive(Debug)]
struct Post {
    path: PathBuf,
    title: String,
    user: String,
    words: usize,
    date: SystemTime,
    duration: String,
    summary: String,
}

impl Default for Post {
    fn default() -> Self {
        Self {
            path: Default::default(),
            title: Default::default(),
            user: Default::default(),
            words: Default::default(),
            date: SystemTime::now(),
            duration: Default::default(),
            summary: Default::default(),
        }
    }
}

fn parse_post(path: &Path) -> Option<Post> {
    let file = File::open(path).unwrap();
    let br = BufReader::new(&file);
    let lines: Vec<String> = br.lines().flatten().collect();

    if lines[0] != "<!-- +++" {
        return None;
    }

    let metadata = file.metadata().unwrap();

    let mut post = Post {
        path: path.to_path_buf(),
        date: metadata.created().unwrap(),
        ..Default::default()
    };

    let mut iter = lines.into_iter();

    for line in &mut iter {
        if line == "<!-- +++" {
            continue;
        }

        if line == "+++ -->" {
            break;
        }

        let (key, value) = line.split_once('=').unwrap();
        let key = key.trim();
        let value = value.trim();

        match key {
            "title" => post.title = value.to_string(),
            "user" => post.user = value.to_string(),
            _ => (),
        }
    }

    //This includes symbols so it's quite inaccurate.
    let mut word_count = 0;
    let mut lines = 0;

    //The summary should probably be done while turning markdown into html.
    //Because everything is interpreted as a string, it includes symbols.
    //"# Title" should be parsed as "Title"

    //There should be a word cap on this i think?
    //Or maybe it should show just the first paragraph?
    let mut summary = String::new();
    for line in &mut iter {
        if lines < 7 {
            lines += 1;
            summary.push_str(&line);
            summary.push('\n');
        }

        let split = line.split(' ');
        word_count += split.count();
    }

    post.summary = summary;

    let string = fs::read_to_string(path).unwrap();
    let tokens = generate_tokens(&string);
    //TODO: Skip over the Metadata +++ .
    let lines: Vec<String> = tokens
        .into_iter()
        .filter_map(|token| {
            if let Token::String(string) = token {
                Some(string)
            } else {
                None
            }
        })
        .collect();

    //TODO: Generate summary fromtokens.

    post.words = word_count;

    //Time to read at 250 WPM.
    let duration = word_count as f32 / 250.0;
    post.duration = if duration < 1.0 {
        String::from("&lt;1 minute read")
    } else {
        format!("{} minute read", duration as u64)
    };

    Some(post)
}

//Collects all markdown files into metadata for the posts page.
fn collect_posts() -> Vec<Post> {
    WalkDir::new("posts")
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            if let Some(ex) = entry.path().extension() {
                if ex == "md" {
                    return parse_post(entry.path());
                }
            }
            None
        })
        .collect()
}

fn generate_post_item(post: Post, template: &str) -> String {
    let mut tokens = generate_tokens(template);

    let mut tree = Vec::new();
    let mut iter = tokens.iter().peekable();

    while let Some(token) = iter.next() {
        match token {
            Token::TemplateStart => {
                let mut i = iter.clone();
                if let Some(Token::String(string)) = i.next() {
                    if let Some(Token::TemplateEnd) = i.next() {
                        iter.advance_by(2);
                        tree.push(Expr::Variable(string.trim().to_string()));
                    }
                }
            }
            Token::ShortcodeStart => {
                todo!();
            }
            Token::String(string) => {
                tree.push(Expr::Text(string.clone()));
            }
            _ => tree.push(Expr::Text(token.to_string())),
        }
    }

    let mut html = String::new();

    for expr in tree {
        match expr {
            Expr::Text(text) => html.push_str(&text),
            Expr::Variable(var) => {
                match &*var {
                    "link" => {
                        html.push_str(&format!(
                            "build/{}.html",
                            post.path.file_stem().unwrap().to_string_lossy()
                        ));
                    }
                    "title" => html.push_str(&post.title),
                    "user" => html.push_str(&post.user),
                    "date" => {
                        // let now = SystemTime::now()
                        //     .duration_since(UNIX_EPOCH)
                        //     .unwrap()
                        //     .as_secs();
                        // html.push_str(&format!("{}", now));
                    }
                    "duration" => html.push_str(&post.duration),
                    "words" => html.push_str(&format!("{} words", post.words)),
                    //A summary should be 2 paragraphs or 400 words.
                    "summary" => html.push_str(&post.summary),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    html.push('\n');

    html
}

//https://alajmovic.com/posts/writing-a-markdown-ish-parser/

//This markdown parser won't support proper markdown.
//No __text__ or _text_ or maybe no **text** or *text*.
//No links with titles [test](http://link.net "title")
//No * or ~ for lists.

fn generate_posts_list(posts: Vec<Post>, template: &str) -> String {
    let mut html = String::new();
    for post in posts {
        html.push_str(&generate_post_item(post, template));
    }

    // println!("{}", html);
    html
}

const POST_TEMPLATE: &str = "templates/post.html";

fn main() {
    //Collect the posts
    let posts = collect_posts();
    //Generate the html based on the template.
    let post_template = fs::read_to_string(POST_TEMPLATE).unwrap();
    let html = generate_posts_list(posts, &post_template);
    //TODO: Inject html into template
    fs::write("posts.html", html);
    // println!("{}", html);
}
//Load Template
//html!("templates/posts.html", posts: Vec<Post>)
//create the variable posts
//read iterator from tempalte and convert to rust
//{% for post in posts %}

//It's going to be really complicated to dynamicly create a variable.

///Convert markdown into a stream of tokens
fn generate_tokens(markdown: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = markdown.chars().peekable();

    let mut string = String::new();
    let mut string_changed = false;

    let mut start = true;

    while let Some(char) = iter.next() {
        if char == '<' {
            let mut i = iter.clone();
            if i.next() == Some('!') && i.next() == Some('-') && i.next() == Some('-') {
                iter.advance_by(3);
                tokens.push(Token::CommentStart);
                continue;
            }
        }

        if char == '-' {
            let mut i = iter.clone();
            if i.next() == Some('-') && i.next() == Some('>') {
                iter.advance_by(2);
                tokens.push(Token::CommentEnd);
                continue;
            }
        }

        match char {
            '+' => {
                let mut i = iter.clone();
                if i.next() == Some('+') && i.next() == Some('+') {
                    iter.advance_by(2);
                    tokens.push(Token::Plus);
                }
            }
            '{' if iter.peek() == Some(&'{') => {
                iter.next();
                tokens.push(Token::TemplateStart);
            }
            '{' if iter.peek() == Some(&'%') => {
                iter.next();
                tokens.push(Token::ShortcodeStart);
            }
            '%' if iter.peek() == Some(&'}') => {
                iter.next();
                tokens.push(Token::ShortcodeEnd);
            }
            '}' if iter.peek() == Some(&'}') => {
                iter.next();
                tokens.push(Token::TemplateEnd);
            }
            '#' if start => {
                let mut i = iter.clone();
                let mut level = 1;

                while let Some('#') = i.next() {
                    level += 1;
                }

                if level > 6 {
                    unreachable!("Invalid heading level.");
                }

                iter.advance_by(level);
                tokens.push(Token::Heading(level as u8));
            }
            '>' => tokens.push(Token::BlockQuote),
            '`' => {
                let mut i = iter.clone();
                if let Some('`') = i.next() {
                    if let Some('`') = i.next() {
                        iter.advance_by(2);
                        tokens.push(Token::CodeBlock);
                        continue;
                    }
                }

                tokens.push(Token::BackTick);
            }
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
            '[' => {
                tokens.push(Token::OpenBracket);
            }
            ']' => tokens.push(Token::CloseBracket),
            '(' => tokens.push(Token::OpenParentheses),
            ')' => tokens.push(Token::CloseParentheses),
            '~' if iter.peek() == Some(&'~') => {
                iter.next();
                tokens.push(Token::Strikethrough);
            }
            '=' => tokens.push(Token::Equal),
            '+' => {
                let mut i = iter.clone();
                if i.next() == Some('+') && i.next() == Some('+') {
                    iter.advance_by(2);
                    tokens.push(Token::Tag);
                }
            }
            '{' => {
                if iter.peek() == Some(&'{') {
                    iter.next();
                    tokens.push(Token::TemplateStart);
                }
            }
            '}' => {
                if iter.peek() == Some(&'}') {
                    iter.next();
                    tokens.push(Token::TemplateEnd);
                }
            }
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

        //Certain tokens like # are only valid if
        //they occur at the start of a string.
        if char == '\n' {
            start = true;
        } else {
            start = false
        }
    }

    //Insert the final string.
    if !string.is_empty() {
        tokens.insert(tokens.len(), Token::String(string));
    }

    tokens
}

fn parse_tokens(tokens: &[Token]) -> Vec<Expr> {
    let mut iter = tokens.iter().peekable();

    let mut ast: Vec<Expr> = Vec::new();

    while let Some(token) = iter.next() {
        if let Some(expr) = expression(token, &mut iter) {
            ast.push(expr);
        }
    }

    ast
}

fn list_item(string: &str) -> Option<&str> {
    let mut chars = string.chars();
    if let Some(first) = chars.next() {
        for char in chars {
            if char == '.' {
                let index = string.find(". ").unwrap();
                if let Some(item) = string.get(index + 2..) {
                    return Some(item);
                };
            }

            if !char.is_numeric() {
                return None;
            }
        }
    }
    None
}

//TODO: Remove all of the string clones.
fn expression(token: &Token, iter: &mut Peekable<Iter<Token>>) -> Option<Expr> {
    match token {
        Token::OpenBracket => {
            let mut i = iter.clone();

            let mut title = String::new();

            if let Some(Token::String(t)) = i.peek() {
                i.next();
                title = t.clone();
            }

            if let Some(Token::CloseBracket) = i.next() {
                if let Some(Token::OpenParentheses) = i.next() {
                    let mut link = String::new();
                    if let Some(Token::String(l)) = i.peek() {
                        i.next();
                        link = l.clone();
                    }

                    if let Some(Token::CloseParentheses) = i.next() {
                        iter.advance_by(3);

                        if !title.is_empty() {
                            iter.advance_by(1);
                        }

                        if !title.is_empty() {
                            iter.advance_by(3);
                        }

                        return Some(Expr::Link(title, link));
                    }
                }
            }

            return None;
        }
        Token::Heading(level) => match iter.next() {
            Some(Token::String(string)) => {
                return Some(Expr::Heading(*level, string.clone()));
            }
            _ => unreachable!("No string after hash!"),
        },
        Token::BlockQuote => {
            let mut level = 1;
            loop {
                match iter.next() {
                    Some(Token::BlockQuote) => {
                        level += 1;
                    }
                    Some(Token::String(string)) => {
                        return Some(Expr::BlockQuote(level, string.clone()));
                    }
                    _ => unreachable!("No string after block quote?"),
                }
            }
        }
        Token::String(string) if string.starts_with('-') => {
            if string.len() > 2 && string.chars().all(|c| c == '-') {
                return Some(Expr::HorizontalRule);
            }
        }
        Token::String(string) if string == "!" => {
            let mut i = iter.clone();
            if let Some(Token::OpenBracket) = i.next() {
                if let Some(Token::String(title)) = i.next() {
                    if let Some(Token::CloseBracket) = i.next() {
                        if let Some(Token::OpenParentheses) = i.next() {
                            if let Some(Token::String(link)) = i.next() {
                                if let Some(Token::CloseParentheses) = i.next() {
                                    iter.advance_by(6);
                                    return Some(Expr::Link(title.clone(), link.clone()));
                                }
                            }
                        }
                    }
                }
            }

            return Some(Expr::Text(String::from("!")));
        }
        Token::String(string) => {
            if let Some(item) = list_item(string) {
                let mut items: Vec<String> = vec![item.to_string()];

                let mut i = iter.clone();

                loop {
                    if i.next() == Some(&Token::CarriageReturn) && i.next() == Some(&Token::NewLine)
                    {
                        if let Some(Token::String(string)) = i.next() {
                            if let Some(list) = list_item(string) {
                                iter.advance_by(3);
                                items.push(list.to_string());
                            } else {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                return Some(Expr::OrderedList(items));
            } else {
                return Some(Expr::Text(string.clone()));
            }
        }
        // **This is some bold text**
        Token::Bold => {
            let mut i = iter.clone();
            if let Some(Token::String(string)) = i.next() {
                if let Some(Token::Bold) = i.next() {
                    iter.advance_by(2);
                    return Some(Expr::Bold(string.clone()));
                }
            }
        }
        // *This is some italic text*
        Token::Italic => {
            let mut i = iter.clone();
            if let Some(Token::String(string)) = i.next() {
                if let Some(Token::Italic) = i.next() {
                    iter.advance_by(2);
                    return Some(Expr::Italic(string.clone()));
                }
            }
        }
        // *This is some striken? text*
        Token::Strikethrough => {
            let mut i = iter.clone();
            if let Some(Token::String(string)) = i.next() {
                if let Some(Token::Strikethrough) = i.next() {
                    iter.advance_by(2);
                    return Some(Expr::Strikethrough(string.clone()));
                }
            }
        }
        Token::BackTick => {
            let mut i = iter.clone();
            if let Some(Token::String(string)) = i.next() {
                if let Some(Token::BackTick) = i.next() {
                    iter.advance_by(2);
                    return Some(Expr::Code(string.clone()));
                }
            }
        }
        Token::CodeBlock => {
            let mut content = Vec::new();
            for token in iter.by_ref() {
                match token {
                    Token::CodeBlock => {
                        return Some(Expr::CodeBlock(content));
                    }
                    Token::String(string) => content.push(string.clone()),
                    _ => (),
                }
            }
        }
        Token::TemplateStart => {
            let mut i = iter.clone();
            if let Some(Token::String(text)) = i.next() {
                if let Some(Token::TemplateEnd) = i.next() {
                    iter.advance_by(2);
                    let name = format!("templates/{}.html", text.trim());
                    let file = if let Ok(file) = std::fs::read_to_string(name) {
                        file
                    } else {
                        format!(
                            "<!-- TEMPLATE FILE '{}.html' DOES NOT EXIST! -->",
                            text.trim()
                        )
                    };
                    return Some(Expr::Template(file));
                }
            }
        }
        Token::Tag => {
            //Tag
            //Key = "Value"
            //Key = "Value"
            //Key = "Value"
            //Tag
        }
        _ => (),
    }
    None
}

fn convert(ast: Vec<Expr>) -> String {
    let mut html = String::new();
    for expr in ast {
        match expr {
            Expr::Heading(level, content) => {
                let h = match level {
                    1 => ("<h1>", r"</h1>"),
                    2 => ("<h2>", r"</h2>"),
                    3 => ("<h3>", r"</h3>"),
                    4 => ("<h4>", r"</h4>"),
                    5 => ("<h5>", r"</h5>"),
                    6 => ("<h6>", r"</h6>"),
                    _ => unreachable!(),
                };

                html.push_str(h.0);
                html.push_str(&content);
                html.push_str(h.1);
            }
            Expr::BlockQuote(level, text) => {
                for _ in 0..level {
                    html.push_str("<blockquote>");
                }
                html.push_str(&text);

                for _ in 0..level {
                    html.push_str("</blockquote>");
                }
            }
            Expr::Bold(text) => html.push_str(&format!("<b>{}</b>", text)),
            Expr::Italic(text) => html.push_str(&format!("<i>{}</i>", text)),
            Expr::OrderedList(items) => {
                html.push_str("<ol>\n");
                for item in items {
                    html.push_str(&format!("\t<li>{}</li>\n", item));
                }
                html.push_str("</ol>");
            }
            Expr::List(_) => todo!(),
            Expr::Link(title, link) => {
                html.push_str(&format!("<a href=\"{}\">{}</a>", link, title))
            }
            Expr::Text(text) => html.push_str(&format!("<p>{}</p>", text)),
            Expr::Strikethrough(text) => html.push_str(&format!("<s>{}</s>", text)),
            //Fenced code blocks don't exist in html so this is kina of dumb.
            Expr::CodeBlock(lines) => {
                html.push_str("<code>\n");
                for line in lines {
                    html.push_str(&format!("{}\n", line));
                }
                html.push_str("</code>");
            }
            Expr::Code(code) => html.push_str(&format!("<code>{}</code>", code)),
            Expr::HorizontalRule => html.push_str("<hr>"),
            Expr::Template(code) => html.push_str(&code),
            _ => (),
        }
        html.push('\n');
    }
    html
}
