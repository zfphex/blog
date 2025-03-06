#![allow(dead_code)]
use mini::*;
use std::{
    ffi::OsStr,
    fs::{metadata, read_to_string},
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

//https://github.com/andresmichel/one-dark-theme
//https://github.com/erremauro/TwoDark
const ONEDARK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\themes\\OneDark.tmTheme");
const TWODARK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\themes\\TwoDark.tmTheme");

const USER_MARKDOWN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\markdown");
const POST_TEMPLATE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\post.html");
const INDEX_TEMPLATE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\index.html");
const INDEX_ITEM_TEMPLATE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\index_item.html");

struct Highlighter {
    ss: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    fn new() -> Self {
        Self {
            ss: SyntaxSet::load_defaults_newlines(),
            theme: ThemeSet::get_theme(TWODARK).unwrap(),
        }
    }
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

#[derive(Debug, Clone)]
struct Template {
    path: &'static str,
    data: String,
    last_write: u64,
}

impl Template {
    fn new(path: &'static str) -> Self {
        Self {
            data: read_to_string(path).unwrap(),
            last_write: metadata(path).unwrap().last_write_time(),
            path,
        }
    }
    fn update(&mut self) -> bool {
        let last_write = metadata(self.path).unwrap().last_write_time();
        if self.last_write != last_write {
            info!("Building template {}", self.path);
            self.last_write = last_write;
            self.data = read_to_string(self.path).unwrap();
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct Post {
    path: PathBuf,
    build_path: PathBuf,
    last_write: u64,
    post_date: String,
    index_date: String,
    title: String,
    summary: String,
    word_count: usize,
    read_time: f32,
}

impl Post {
    const BUILD_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site");

    fn new(path: &Path, post_template: &Template, highlighter: &mut Highlighter) -> Option<Self> {
        info!("Building post {}", path.to_string_lossy());

        // Convert "markdown/example.md" to "build/example.html"
        let build_path = {
            let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
            name.pop();
            name.pop();
            name.push_str("html");
            PathBuf::from(Self::BUILD_PATH).join(name)
        };
        let md = read_to_string(&path).unwrap();
        let mut title = String::new();
        let mut html = String::new();
        let mut post_date = String::new();
        let mut index_date = String::new();
        let mut summary = String::new();

        const START_SEPERATOR: &str = "<!--";
        const END_SEPERATOR: &str = "-->";
        //Find the opening `<!--`
        let body = if md.get(..START_SEPERATOR.len()) == Some(START_SEPERATOR) {
            let Some(end) = md[START_SEPERATOR.len()..].find(END_SEPERATOR) else {
                error!("Invalid metadata {}", path.display());
                return None;
            };

            for line in md[START_SEPERATOR.len()..end + END_SEPERATOR.len()].split('\n') {
                if let Some((k, v)) = line.split_once(':') {
                    let v = v.trim();
                    match k {
                        "title" => {
                            title.clear();
                            title.push_str(v)
                        }
                        "summary" => {
                            summary.clear();
                            summary.push_str(v)
                        }
                        "date" => {
                            let splits: Vec<&str> = v.split('/').collect();
                            if splits.len() != 3 {
                                error!("Invalid date: '{}' {}", v, &path.display());
                                continue;
                            }
                            let Ok(d) = &splits[0].parse::<usize>() else {
                                continue;
                            };
                            let Ok(m) = &splits[1].parse::<usize>() else {
                                continue;
                            };
                            let Ok(year) = &splits[2].parse::<usize>() else {
                                error!("Invalid date: '{}' {}", v, &path.display());
                                continue;
                            };
                            if *year < 1000 {
                                error!("Invalid year: '{}' {}", v, &path.display());
                                continue;
                            }
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

                            index_date = format!("{day} {month}, {year}");
                            post_date = format!("{day} of {month}, {year}");
                        }
                        _ => continue,
                    }
                }
            }

            //Get the user content excluding the metadata.
            //Add the start '<!--' then the ending position plus '-->'.
            &md[START_SEPERATOR.len() + end + END_SEPERATOR.len()..]
        } else {
            &md
        };

        let mut pathbuf = path.to_path_buf();
        pathbuf.set_extension("html");

        let word_count = body.split(|c: char| c.is_whitespace()).count();
        let read_time = word_count as f32 / 250.0;

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

        html.clear();
        pulldown_cmark::html::push_html(&mut html, parser);

        //Generate the post using the metadata and html.
        let post = post_template
            .data
            .replace("<!-- title -->", &title)
            .replace("<!-- date -->", &post_date)
            .replace("<!-- content -->", &html);

        std::fs::write(&build_path, &post).unwrap();

        Some(Self {
            path: PathBuf::from(path),
            last_write: metadata(path).unwrap().last_write_time(),
            build_path,
            post_date,
            index_date,
            title,
            summary,
            word_count,
            read_time,
        })
    }
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
    fn update(&mut self, post_template: &Template, highlighter: &mut Highlighter) -> bool {
        let last_write = metadata(&self.path).unwrap().last_write_time();
        if last_write != self.last_write {
            info!("Building post {}", self.path.to_string_lossy());
            if let Some(new) = Self::new(&self.path, post_template, highlighter) {
                *self = new;
            }
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct Posts {
    posts: Vec<Post>,
}

impl Posts {
    fn new(
        file_watcher: &FileWatcher,
        post_template: &Template,
        highlighter: &mut Highlighter,
    ) -> Self {
        Self {
            posts: file_watcher
                .files
                .iter()
                .flat_map(|file| Post::new(Path::new(&file.path), post_template, highlighter))
                .collect(),
        }
    }
}

struct Index {}

impl Index {
    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site\\index.html");

    fn new(index_item_template: &Template, index_template: &Template, posts: &Posts) -> Self {
        let index_template = &index_template.data;
        let item_template = &index_item_template.data;
        let i = index_template
            .find("<!-- posts -->")
            .expect("Couldn't find <!-- posts -->");

        // TODO: Implement sorting by d/m/y
        // files.sort_by(|_a, _b| {
        //     Ordering::Equal
        // });

        let mut template = index_template.replace("<!-- posts -->", "");

        for post in &posts.posts {
            let item = item_template
                .replace("<!-- title -->", &post.title)
                .replace("<!-- summary -->", &post.summary)
                .replace("<!-- date -->", &post.index_date)
                .replace("<!-- read_time -->", &post.read_time())
                .replace("<!-- word_count -->", &post.word_count())
                .replace(
                    "<!-- link -->",
                    post.build_path.file_name().unwrap().to_str().unwrap(),
                );

            template.insert_str(i, &item);
        }

        std::fs::write(Self::PATH, template).unwrap();

        Self {}
    }

    fn update(&mut self, index_item_template: &Template, index_template: &Template, posts: &Posts) {
        info!("Updating Index");
        *self = Self::new(index_item_template, index_template, posts);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FileWatcher {
    files: Vec<DirEntry>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            files: walkdir(USER_MARKDOWN_PATH, 1)
                .into_iter()
                .flatten()
                .filter(|file| !file.is_folder)
                .filter(|file| file.extension() == Some(OsStr::new("md")))
                .collect(),
        }
    }

    fn update(&mut self) -> bool {
        let fw = FileWatcher::new();
        if self != &fw {
            *self = fw;
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
            true
        } else {
            false
        }
    }
}

fn main() {
    //Start tailwind
    #[cfg(feature = "tailwind")]
    std::thread::spawn(|| {
        //tailwindcss -i templates/input.css -o styles/tailwind.css --watch
        let mut command = std::process::Command::new("tailwindcss")
            .args(&[
                "--cwd",
                "templates/",
                // "-i",
                // "templates/input.css",
                "-o",
                "../site/assets/tailwind.css",
                "--watch",
            ])
            .spawn()
            .unwrap();
        command.wait().unwrap();
    });

    //For "reasons" site/ is my github pages repo.
    assert!(Path::new("site").exists());

    let mut highlighter = Highlighter::new();
    let mut post_template = Template::new(&POST_TEMPLATE_PATH);
    let mut index_template = Template::new(&INDEX_TEMPLATE_PATH);
    let mut index_item_template = Template::new(&INDEX_ITEM_TEMPLATE_PATH);

    let mut file_watcher = FileWatcher::new();
    let mut posts = Posts::new(&file_watcher, &post_template, &mut highlighter);
    let mut index = Index::new(&index_item_template, &index_template, &posts);

    // Relationships
    // Index Template -> Index
    // Index Item Template -> Index
    // Post Template -> Posts -> Index

    // Files -> Posts -> Index

    info!("Started watching files");

    loop {
        //Index Template -> Index
        if index_template.update() {
            index.update(&index_item_template, &index_template, &posts);
        }

        //Index Item Template -> Index
        if index_item_template.update() {
            index.update(&index_item_template, &index_template, &posts);
        }

        //Post Template -> Posts -> Index
        if post_template.update() {
            posts = Posts::new(&file_watcher, &post_template, &mut highlighter);
            index.update(&index_item_template, &index_template, &posts);
        }

        //Files -> Posts -> Index
        if file_watcher.update() {
            //TODO: Only add or remove old/new posts. Not everything.
            posts = Posts::new(&file_watcher, &post_template, &mut highlighter);
            index.update(&index_item_template, &index_template, &posts);
        }

        //Posts -> Index
        let mut post_updated = false;
        for post in &mut posts.posts {
            if post.update(&post_template, &mut highlighter) {
                post_updated = true;
            }
        }

        if post_updated {
            index.update(&index_item_template, &index_template, &posts);
        }

        std::thread::sleep(Duration::from_millis(32));
    }
}
