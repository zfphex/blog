use mini::*;
use std::{
    fs,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Duration,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    html::{append_highlighted_html_for_styled_line, IncludeBackground},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use winwalk::*;

const POLL_DURATION: Duration = Duration::from_millis(66);

const MARKDOWN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\markdown");
const BUILD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site");
const INDEX: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site\\index.html");
const TEMPLATE_INDEX: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\index.html");
const TEMPLATE_POST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\post.html");
const TEMPLATE_ITEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\item.html");
// const CSS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site\\assets\\style.css");

struct Highlighter {
    ss: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    fn highlight(&mut self, lang: &str, code: &str) -> String {
        profile!();
        let syntax = self
            .ss
            .find_syntax_by_token(&lang)
            .unwrap_or_else(|| self.ss.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut html = String::new();

        for line in LinesWithEndings::from(&code) {
            let regions = highlighter.highlight_line(line, &self.ss).unwrap();
            append_highlighted_html_for_styled_line(&regions[..], IncludeBackground::No, &mut html)
                .unwrap();
        }
        html
    }
}

#[derive(Debug, Default)]
struct File {
    pub path: PathBuf,
    pub build_path: PathBuf,
    pub last_write: SystemTime,

    pub md: String,
    pub html: String,
    pub post: String,

    ///Date displayed on post.
    pub post_date: String,
    ///Date displayed on index.
    pub index_date: String,

    //Really these should be &'a str, but self referencing structs in Rust don't work.
    pub title: String,
    pub summary: String,

    pub word_count: usize,
    pub read_time: f32,
}

impl File {
    fn word_count(&self) -> String {
        if self.word_count != 1 {
            format!("{} words", self.word_count)
        } else {
            String::from("1 word")
        }
    }
    fn read_time(&self) -> String {
        if self.read_time < 1.0 {
            String::from("&lt;1 minute read")
        } else {
            format!("{} minute read", self.read_time as usize)
        }
    }
}

struct Template {
    pub path: &'static str,
    pub data: String,
    pub last_write: u64,
}

impl Template {
    pub fn update(&mut self, rebuild: &mut bool) {
        let last_write = fs::metadata(self.path).unwrap().last_write_time();
        if self.last_write != last_write {
            *rebuild = true;
            info!("Updated {}", self.path);
            self.last_write = last_write;
            self.data = fs::read_to_string(self.path).unwrap();
        }
    }
}

fn template(path: &'static str) -> Template {
    //read_to_string reads the metadata. Then we read it again 🙄.
    //Functions like default_read_to_end are private so we can't write our own.
    let data = fs::read_to_string(path).unwrap();
    let last_write = fs::metadata(path).unwrap().last_write_time();
    Template {
        path,
        data,
        last_write,
    }
}

///Read post metadata, highlight code and create html file.
fn metadata(file: &mut File, template: &str, highlighter: &mut Highlighter) {
    const SEPERATOR: &str = "~~~";

    //Find the opening `~~~`.
    let len = SEPERATOR.len();
    let body = if file.md.get(..len) == Some(SEPERATOR) {
        let Some(end) = file.md[len..].find(SEPERATOR) else {
            return error!("Invalid metadata {}", file.path.display());
        };

        for line in file.md[len..end + len].split('\n') {
            if let Some((k, v)) = line.split_once(':') {
                let v = v.trim();
                match k {
                    "title" => {
                        file.title.clear();
                        file.title.push_str(v)
                    }
                    "summary" => {
                        file.summary.clear();
                        file.summary.push_str(v)
                    }
                    "date" => {
                        let splits: Vec<&str> = v.split('/').collect();
                        if splits.len() != 3 {
                            error!("Invalid date: {} {:?}", v, &file.path);
                            continue;
                        }
                        let d = &splits[0].parse::<usize>().unwrap();
                        let m = &splits[1].parse::<usize>().unwrap();
                        let year = &splits[2].parse::<usize>().unwrap();
                        let month = match m {
                            1 => "January",
                            2 => "February",
                            3 => "March",
                            4 => "April",
                            5 => "May",
                            6 => "June",
                            7 => "July",
                            8 => "August",
                            9 => "September",
                            10 => "October",
                            11 => "November",
                            12 => "December",
                            _ => unreachable!(),
                        };

                        //Ordinal suffix.
                        let i = d % 10;
                        let j = d % 100;
                        let day = match i {
                            1 if j != 11 => format!("{d}st"),
                            2 if j != 12 => format!("{d}nd"),
                            3 if j != 13 => format!("{d}rd"),
                            _ => format!("{d}th"),
                        };

                        //Can't wait for the Y3K problem.
                        file.index_date = format!("{day} {month}, 20{year}");
                        file.post_date = format!("{day} of {month}, 20{year}");
                    }
                    _ => continue,
                }
            }
        }

        //Get the user content excluding the metadata.
        //Add the start '~~~' then the ending position plus '~~~'.
        &file.md[len + end + len..]
    } else {
        &file.md
    };

    let mut pathbuf = file.path.clone();
    pathbuf.set_extension("html");

    file.word_count = body.split(|c: char| c.is_whitespace()).count();
    file.read_time = file.word_count as f32 / 250.0;

    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

    let parser = Parser::new_ext(
        &body,
        Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS,
    );

    let mut lang = String::new();
    let mut code = false;

    let parser = parser.map(|event| match event {
        Event::Start(tag) => match tag {
            Tag::CodeBlock(info) => match info {
                CodeBlockKind::Fenced(fenced) => {
                    code = true;
                    lang = fenced.to_string();
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(fenced)))
                }
                CodeBlockKind::Indented => Event::Start(Tag::CodeBlock(info)),
            },
            _ => Event::Start(tag),
        },
        Event::End(tag) => match tag {
            Tag::CodeBlock(info) => {
                code = false;
                Event::End(Tag::CodeBlock(info))
            }
            _ => Event::End(tag),
        },
        Event::Text(text) if code => {
            if &lang == "math" {
                todo!();
            } else {
                Event::Html(highlighter.highlight(&lang, &text).into())
            }
        }
        _ => event,
    });

    file.html.clear();
    pulldown_cmark::html::push_html(&mut file.html, parser);

    //Generate the post using the metadata and html.
    file.post = template
        .replace("<!-- title -->", &file.title)
        .replace("<!-- date -->", &file.post_date)
        .replace("<!-- content -->", &file.html);

    fs::write(&file.build_path, &file.post).unwrap();
}

fn main() {
    // defer_results!();

    //For "reasons" site/ is my github pages repo.
    assert!(Path::new("site").exists());

    let mut files: Vec<File> = Vec::new();
    let mut walk: Vec<DirEntry>;
    let mut post = template(TEMPLATE_POST);
    let mut index = template(TEMPLATE_INDEX);
    let mut item = template(TEMPLATE_ITEM);

    let ts = ThemeSet::load_defaults();
    let mut highlighter = Highlighter {
        ss: SyntaxSet::load_defaults_newlines(),
        theme: ts.themes["base16-ocean.dark"].clone(),
    };

    let mut rebuild = false;
    let md = std::ffi::OsStr::new("md");
    #[allow(unused)]
    let html = std::ffi::OsStr::new("html");

    loop {
        post.update(&mut rebuild);
        index.update(&mut rebuild);
        item.update(&mut rebuild);

        walk = walkdir(MARKDOWN, 1)
            .into_iter()
            .flatten()
            .filter(|file| !file.is_folder)
            .filter(|file| file.extension() == Some(&md))
            .collect();

        if walk.len() != files.len() {
            files.clear();

            //NOTE: I'd rather do this manually for now, I don't like force deleting files.

            //Delete all the old html files.
            // for file in walkdir(BUILD, 1)
            //     .into_iter()
            //     .flatten()
            //     .filter(|file| !file.is_folder)
            //     .filter(|file| file.extension() == Some(&html))
            // {
            //     info!("Removed file {}", &file.path);
            //     fs::remove_file(&file.path).unwrap();
            // }
        }

        //TODO: It seems like last_write is invalid for symlinks...
        for file in walk {
            let f = files
                .iter_mut()
                .find(|f| f.path.as_path() == Path::new(&file.path));

            match f {
                //File has changes. Currenty there is no way to read a file without re-allocating.
                Some(f) if f.last_write != file.last_write => {
                    info!("Updated contents of {}", f.path.display());

                    rebuild = true;
                    f.last_write = file.last_write;
                    f.md = fs::read_to_string(&f.path).unwrap();
                    metadata(f, &post.data, &mut highlighter);
                }
                //File exists and has no changes.
                Some(_) => {}
                //File is new.
                None => {
                    rebuild = true;
                    info!("Adding file {}", file.path);
                    let path = PathBuf::from(&file.path);
                    let mut file = File {
                        md: fs::read_to_string(&file.path).unwrap(),
                        html: String::new(),
                        post: String::new(),
                        // Convert "markdown/example.md" to "build/example.html"
                        build_path: {
                            let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
                            name.pop();
                            name.pop();
                            name.push_str("html");
                            PathBuf::from(BUILD).join(name)
                        },
                        path,
                        last_write: file.last_write,
                        post_date: String::new(),
                        index_date: String::new(),
                        title: String::new(),
                        summary: String::new(),
                        word_count: 0,
                        read_time: 0.0,
                    };
                    metadata(&mut file, &post.data, &mut highlighter);
                    files.push(file);
                }
            }
        }

        if rebuild {
            let index = &index.data;
            let item = &item.data;
            let i = index
                .find("<!-- posts -->")
                .expect("Couldn't find <!-- posts -->");

            let mut template = index.replace("<!-- posts -->", "");

            // TODO: Implement sorting by d/m/y
            // files.sort_by(|_a, _b| {
            //     Ordering::Equal
            // });

            for file in &files {
                let item = item
                    .replace("<!-- title -->", &file.title)
                    .replace("<!-- summary -->", &file.summary)
                    .replace("<!-- date -->", &file.index_date)
                    .replace("<!-- read_time -->", &file.read_time())
                    .replace("<!-- word_count -->", &file.word_count())
                    .replace(
                        "<!-- link -->",
                        file.build_path.file_name().unwrap().to_str().unwrap(),
                    );

                template.insert_str(i, &item);
            }

            info!("Compiled index {}", INDEX);
            fs::write(INDEX, template).unwrap();
            rebuild = false;
        }

        std::hint::spin_loop();
        std::thread::sleep(POLL_DURATION);
    }
}
