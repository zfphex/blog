#![feature(allocator_api)]
use chrono::{DateTime, Datelike, FixedOffset, Utc};
use mini::*;
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const POLL_DURATION: Duration = Duration::from_millis(250);

const MARKDOWN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\markdown");
const BUILD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site");

const INDEX: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site\\index.html");

const TEMPLATE_INDEX: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\index.html");
const TEMPLATE_POST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\post.html");
const TEMPLATE_ITEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\templates\\item.html");

const CSS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site\\assets\\style.css");
const CSS_MIN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\site\\assets\\style-min.css");

#[cfg(feature = "minify")]
fn minify(html: &str) -> Vec<u8> {
    let mut cfg = minify_html::Cfg::spec_compliant();
    cfg.keep_closing_tags = true;
    cfg.keep_html_and_head_opening_tags = true;
    cfg.minify_css = true;
    cfg.minify_js = true;
    minify_html::minify(html.as_bytes(), &cfg)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Hash(blake3::Hash);
impl Default for Hash {
    fn default() -> Self {
        Self(blake3::Hash::from_bytes([0; 32]))
    }
}

fn hash(bytes: &[u8]) -> Result<Hash, Box<dyn Error>> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    Ok(Hash(hasher.finalize()))
}

struct List {
    pub posts: HashMap<PathBuf, Post>,
    pub highligher: Highlighter,
}

impl List {
    pub fn new() -> Self {
        Self {
            posts: HashMap::new(),
            highligher: Highlighter::new(),
        }
    }
    pub fn update(&mut self, templates: &mut Templates) -> Result<(), Box<dyn Error>> {
        let files: Vec<PathBuf> = fs::read_dir(MARKDOWN)?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| matches!(path.extension().and_then(OsStr::to_str), Some("md")))
            .collect();

        let mut rebuild_list = false;
        let (post_template, _) = &templates.post;

        let posts = std::mem::take(&mut self.posts);
        self.posts = posts
            .into_iter()
            .filter_map(|(k, v)| {
                if files.contains(&k) {
                    Some((k, v))
                } else {
                    rebuild_list = true;
                    match fs::remove_file(&v.build_path) {
                        Ok(_) => info!("Removed: {:?}", v.build_path),
                        Err(err) => warn!("Failed to removed: {:?}\n{err}", v.build_path),
                    };
                    None
                }
            })
            .collect();

        //Add the new posts
        for path in files {
            //Generate a hash for the file.
            let file = fs::read_to_string(&path)?;
            let hash = hash(file.as_bytes())?;

            if let Some(post) = self.posts.get_mut(&path) {
                //File is out of date.
                if hash != post.hash {
                    rebuild_list = true;
                    match Post::new(&post_template, file, &path, hash, &self.highligher) {
                        Ok(p) => *post = p,
                        Err(err) => warn!("Failed to compile: {path:?}\n{err}"),
                    };
                }
            } else {
                //File is new.
                rebuild_list = true;
                match Post::new(&post_template, file, &path, hash, &self.highligher) {
                    Ok(post) => {
                        self.posts.insert(path, post);
                    }
                    Err(err) => warn!("Failed to compile: {path:?}\n{err}"),
                };
            }
        }

        if rebuild_list {
            self.build_list(templates)?;
        }

        Ok(())
    }
    pub fn build_list(&self, templates: &mut Templates) -> Result<(), Box<dyn Error>> {
        let (list_template, _) = &templates.index;
        let (list_item_template, _) = &templates.item;

        let index = list_template
            .find("<!-- posts -->")
            .ok_or("Couldn't find <!-- posts -->")?;

        let mut template = list_template.replace("<!-- posts -->", "");

        let mut posts: Vec<&Post> = self.posts.values().collect();
        posts.sort_by_key(|post| post.metadata.date);

        for post in posts {
            let metadata = &post.metadata;
            let (day, month, year) = metadata.date();
            let list_item = list_item_template
                .replace("~link~", &metadata.link_path)
                .replace("<!-- title -->", &metadata.title)
                .replace("<!-- date -->", &format!("{day} {month} {year}"))
                .replace("<!-- read_time -->", &metadata.read_time())
                .replace("<!-- word_count -->", &metadata.word_count())
                .replace("<!-- summary -->", &metadata.summary);

            template.insert_str(index, &list_item);
        }

        #[cfg(feature = "minify")]
        let template = minify(&template);

        fs::write(INDEX, template)?;
        info!("Compiled: {}", INDEX);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub summary: String,
    pub date: DateTime<FixedOffset>,
    pub link_path: String,
    pub real_path: PathBuf,
    pub word_count: usize,
    pub read_time: f32,
    pub end_position: usize,
}

impl Metadata {
    pub fn new(file: &str, path: &Path) -> Result<Self, Box<dyn Error>> {
        let config = file.get(3..).ok_or("Invalid metadata")?.trim_start();

        let mut title = String::new();
        let mut summary = String::new();

        let creation_date: DateTime<Utc> = fs::metadata(path)?.created()?.into();
        let mut date = creation_date.into();

        let mut end = 0;

        if let Some(e) = config.find("~~~") {
            for line in config.split('\n') {
                if line.starts_with("~~~") {
                    //NOTE: Config is offset by 4(zero index as 3)!
                    end = 4 + e + line.len();
                    break;
                }

                if let Some((k, v)) = line.split_once(':') {
                    let v = v.trim();
                    match k {
                        "title" => title = v.to_string(),
                        "summary" => summary = v.to_string(),
                        "date" => {
                            date = DateTime::parse_from_str(
                                &format!("{v} 00:00"),
                                "%d/%m/%Y %z %H:%M",
                            )?;
                        }
                        _ => continue,
                    }
                }
            }
        }

        let mut pathbuf = path.to_path_buf();
        pathbuf.set_extension("html");

        //Rough estimate of the word count. Doesn't actually count alphanumerically.
        let word_count = file[end..].split(|c: char| c.is_whitespace()).count();

        Ok(Metadata {
            title,
            summary,
            date,
            link_path: pathbuf
                .file_name()
                .ok_or("file_name")?
                .to_str()
                .ok_or("to_str")?
                .to_string(),
            real_path: path.to_path_buf(),
            read_time: word_count as f32 / 250.0,
            word_count,
            end_position: end,
        })
    }
    pub fn word_count(&self) -> String {
        if self.word_count != 1 {
            format!("{} words", self.word_count)
        } else {
            String::from("1 word")
        }
    }
    pub fn read_time(&self) -> String {
        if self.read_time < 1.0 {
            String::from("&lt;1 minute read")
        } else {
            format!("{} minute read", self.read_time as usize)
        }
    }
    pub fn date(&self) -> (String, String, i32) {
        let month = match self.date.month() {
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
        let day = self.date.day();
        let i = day % 10;
        let j = day % 100;
        let day = match i {
            1 if j != 11 => format!("{day}st"),
            2 if j != 12 => format!("{day}nd"),
            3 if j != 13 => format!("{day}rd"),
            _ => format!("{day}th"),
        };

        (day, month.to_string(), self.date.year())
    }
}

use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    html::{append_highlighted_html_for_styled_line, IncludeBackground},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

pub struct Highlighter {
    pub ss: SyntaxSet,
    pub buffer: String,
    pub theme: Theme,
}

impl Highlighter {
    pub fn new() -> Self {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = ts.themes["base16-ocean.dark"].clone();

        Self {
            ss,
            theme,
            buffer: String::new(),
        }
    }

    pub fn highlight(&self, lang: &str, code: &str) -> String {
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

#[derive(Debug)]
pub struct Post {
    pub metadata: Metadata,
    pub build_path: PathBuf,
    pub hash: Hash,
}

impl Post {
    pub fn new(
        post_template: &str,
        file: String,
        path: &Path,
        hash: Hash,
        highligher: &Highlighter,
    ) -> Result<Self, Box<dyn Error>> {
        use pulldown_cmark::*;

        //Read the markdown file.
        // let file = fs::read_to_string(path)?;
        let metadata = Metadata::new(&file, path)?;
        let file = &file[metadata.end_position..].trim_start();

        let options = Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TABLES
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS;

        let parser = Parser::new_ext(file, options);
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
                // let text = if &lang == "math" {
                // } else {
                //     highligher.highlight(&lang, &text)
                // };
                let text = highligher.highlight(&lang, &text);
                Event::Html(text.into())
            }
            _ => event,
        });

        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);

        //Generate the post using the metadata and html.
        let (day, month, year) = metadata.date();
        let post = post_template
            .replace("<!-- title -->", &metadata.title)
            .replace("<!-- date -->", &format!("{day} of {month}, {year}"))
            .replace("<!-- content -->", &html);

        //Convert "markdown/example.md" to "build/example.html"
        let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
        name.pop();
        name.pop();
        name.push_str("html");
        let build_path = PathBuf::from(BUILD).join(name);

        #[cfg(feature = "minify")]
        let post = minify(&post);

        fs::write(&build_path, post)?;
        info!("Created new post: {path:?}");

        Ok(Self {
            metadata,
            build_path,
            hash,
        })
    }
}

struct Templates {
    pub post: (String, Hash),
    pub index: (String, Hash),
    pub item: (String, Hash),
}

impl Templates {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            post: {
                let file = fs::read_to_string(TEMPLATE_POST)?;
                let hash = hash(file.as_bytes())?;
                (file, hash)
            },
            index: {
                let file = fs::read_to_string(TEMPLATE_INDEX)?;
                let hash = hash(file.as_bytes())?;
                (file, hash)
            },
            item: {
                let file = fs::read_to_string(TEMPLATE_ITEM)?;
                let hash = hash(file.as_bytes())?;
                (file, hash)
            },
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    //This will fail if the folder already exists.
    let _ = fs::create_dir(BUILD);

    info!("Watching files in {:?}", Path::new(MARKDOWN));

    let mut t = Templates::new()?;
    let mut posts = List::new();
    let mut css = (String::new(), Hash::default());

    let mut first = Some(Instant::now());

    loop {
        //Make sure the templates and `style-min.css` are up to date.
        if build(CSS, &mut css, &mut false)? {
            #[cfg(feature = "minify")]
            let css = minify(&css);
            fs::write(&CSS_MIN, &css.0)?;
        }

        //Re-build the posts.
        if build(TEMPLATE_POST, &mut t.post, &mut false)? {
            //Delete the old hashes and List::update() will do the work.
            for (_, v) in posts.posts.iter_mut() {
                v.hash = Hash::default();
            }
        }

        let mut rebuild_list = false;

        build(TEMPLATE_INDEX, &mut t.index, &mut rebuild_list)?;
        build(TEMPLATE_ITEM, &mut t.item, &mut rebuild_list)?;

        if rebuild_list {
            posts.build_list(&mut t)?;
        }

        posts.update(&mut t)?;

        if first.is_some() {
            info!("Finished in {:?}", first.unwrap().elapsed());
            first = None;
        }

        unsafe { core::arch::x86_64::_mm_pause() };

        std::thread::sleep(POLL_DURATION);
    }
}

pub fn build(
    path: &str,
    template: &mut (String, Hash),
    rebuild: &mut bool,
) -> Result<bool, Box<dyn Error>> {
    let file = fs::read_to_string(path)?;
    let new_hash = hash(file.as_bytes())?;
    if new_hash != template.1 {
        info!("Compiled: {:?}", path);
        template.0 = file;
        template.1 = new_hash;
        *rebuild = true;
        Ok(true)
    } else {
        Ok(false)
    }
}
