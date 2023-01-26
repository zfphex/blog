#![feature(hash_drain_filter)]
use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fs::{self, File},
    io::Cursor,
    path::{Path, PathBuf},
    time::Duration,
};

const MARKDOWN_PATH: &str = "markdown";
const TEMPLATE_PATH: &str = "templates";
const BUILD_PATH: &str = "build";
const POLL_DURATION: Duration = Duration::from_millis(250);

mod hex;
mod html;

fn now() -> String {
    Local::now().time().format("%H:%M:%S").to_string()
}

fn minify(html: &str) -> Vec<u8> {
    let mut cfg = minify_html::Cfg::spec_compliant();
    cfg.keep_html_and_head_opening_tags = true;
    cfg.minify_css = true;
    cfg.minify_js = true;
    minify_html::minify(html.as_bytes(), &cfg)
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        print!("\x1b[90m{} \x1b[94mINFO\x1b[0m ", now());
        println!($($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        print!("\x1b[90m{} \x1b[93mWARN\x1b[0m '{}:{}:{}' ", now(), file!(), line!(), column!());
        println!($($arg)*);
    }};
}

fn hash(path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    use blake3::*;

    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    let map = unsafe {
        memmap2::MmapOptions::new()
            .len(file_size as usize)
            .map(&file)?
    };

    let cursor = Cursor::new(map);
    let mut hasher = Hasher::new();
    hasher.update(cursor.get_ref());

    let mut output = hasher.finalize_xof();
    let mut block = [0; blake3::guts::BLOCK_LEN];
    let mut len = 32;
    let mut hex = String::new();

    while len > 0 {
        output.fill(&mut block);
        let hex_str = hex::encode(&block[..]);
        let take_bytes = std::cmp::min(len, block.len() as u64);
        hex.push_str(&hex_str[..2 * take_bytes as usize]);
        len -= take_bytes;
    }

    Ok(hex)
}

struct Posts {
    pub posts: HashMap<PathBuf, Post>,
    pub list_template: String,
    pub list_item_template: String,
    pub post_template: String,
}

impl Posts {
    pub fn new() -> Self {
        Self {
            posts: HashMap::new(),
            list_template: fs::read_to_string("templates/post_list.html").unwrap(),
            list_item_template: fs::read_to_string("templates/post_list_item.html").unwrap(),
            post_template: fs::read_to_string("templates/post.html").unwrap(),
        }
    }
    pub fn update_templates(&mut self) {
        self.list_template = fs::read_to_string("templates/post_list.html").unwrap();
        self.list_item_template = fs::read_to_string("templates/post_list_item.html").unwrap();
        self.post_template = fs::read_to_string("templates/post.html").unwrap();
    }
    pub fn build_post(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        match Post::new(&self.post_template, path) {
            Ok(post) => {
                info!("Compiled: {path:?}");
                post.write()?;
                self.posts.insert(path.to_path_buf(), post);
            }
            Err(err) => warn!("Failed to compile: {path:?}\n{err}"),
        };
        Ok(())
    }
    pub fn build_list(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Compiled: \"build\\\\post_list.html\"");
        let index = self
            .list_template
            .find("<!-- posts -->")
            .ok_or("Couldn't find <!-- posts -->")?;
        let mut template = self.list_template.replace("<!-- posts -->", "");

        let mut posts: Vec<&Post> = self.posts.values().collect();
        posts.sort_by_key(|post| post.metadata.date);

        for post in posts {
            let metadata = &post.metadata;
            let (day, month, year) = metadata.date();
            let list_item = self
                .list_item_template
                .replace("~link~", &metadata.link_path)
                .replace("<!-- title -->", &metadata.title)
                .replace("<!-- date -->", &format!("{day} {month} {year}"))
                .replace("<!-- read_time -->", &metadata.read_time())
                .replace("<!-- word_count -->", &metadata.word_count())
                .replace("<!-- summary -->", &metadata.summary);

            template.insert_str(index, &list_item);
        }

        let template = minify(&template);

        fs::write("build/post_list.html", template)?;

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

#[derive(Debug)]
pub struct Post {
    pub html: String,
    pub metadata: Metadata,
    pub build_path: PathBuf,
}

impl Post {
    pub fn new(post_template: &str, path: &Path) -> Result<Self, Box<dyn Error>> {
        use pulldown_cmark::*;

        //Read the markdown file.
        let file = fs::read_to_string(path)?;

        let metadata = Metadata::new(&file, path)?;
        let file = &file[metadata.end_position..].trim_start();

        //TODO: Add syntax highlighting to the code blocks.

        //Convert the markdown to html.
        let parser = Parser::new_ext(file, Options::all());
        let mut html = String::new();
        // html::push_html(&mut html, parser);
        crate::html::push_html(&mut html, parser);

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
        let path = PathBuf::from(BUILD_PATH).join(name);

        Ok(Self {
            html: post,
            metadata,
            build_path: path,
        })
    }
    pub fn write(&self) -> std::io::Result<()> {
        let html = minify(&self.html);
        fs::write(&self.build_path, html)
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
    ss: SyntaxSet,
    buffer: String,
    theme: Theme,
    language: String,
}

impl Highlighter {
    //TODO: Create custom theme.
    pub(crate) fn new() -> Self {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = ts.themes["base16-ocean.dark"].clone();

        Self {
            ss,
            theme,
            buffer: String::new(),
            language: String::new(),
        }
    }
    pub fn push_str(&mut self, str: &str) {
        self.buffer.push_str(str);
    }
    pub fn highlight(&mut self) -> String {
        let syntax = self
            .ss
            .find_syntax_by_token(&self.language)
            .unwrap_or_else(|| self.ss.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut html = String::new();

        let lines = std::mem::take(&mut self.buffer);
        for line in LinesWithEndings::from(&lines) {
            let regions = highlighter.highlight_line(line, &self.ss).unwrap();
            append_highlighted_html_for_styled_line(&regions[..], IncludeBackground::No, &mut html)
                .unwrap();
        }

        html
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    //Make sure the build folder exists.
    let _ = fs::create_dir(BUILD_PATH);

    info!("Watching files in {:?}", Path::new(MARKDOWN_PATH));

    let mut posts = Posts::new();
    let mut files: HashMap<PathBuf, String> = HashMap::new();

    loop {
        let new_files: Vec<PathBuf> = fs::read_dir(MARKDOWN_PATH)?
            .chain(fs::read_dir(TEMPLATE_PATH)?)
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(OsStr::to_str),
                    Some("md") | Some("html")
                )
            })
            .collect();

        //Make sure 'physically' deleted files are removed from memory.
        if new_files.len() != files.len() {
            files.drain_filter(|k, _| {
                if !new_files.contains(k) {
                    info!("Removed: {:?}", k);
                    true
                } else {
                    false
                }
            });
        }

        let mut outdated_files: Vec<PathBuf> = new_files
            .into_iter()
            .filter_map(|path| {
                //Generate a hash for the file.
                let hash = hash(&path).unwrap_or_default();

                match files.insert(path.clone(), hash.clone()) {
                    //File is out of date.
                    Some(old_hash) if hash != old_hash => Some(path),
                    //File is new.
                    None => Some(path),
                    //File is up to date.
                    _ => None,
                }
            })
            .collect();

        //Sort "html" files first. If a template needs updating, it's wasteful to rebuild everything twice.
        outdated_files.sort_by(|a, b| a.extension().cmp(&b.extension()));

        for file in &outdated_files {
            match file.extension().and_then(OsStr::to_str) {
                Some("html") => {
                    info!("Re-building templates.");
                    posts.update_templates();

                    let md: Vec<&PathBuf> = files
                        .keys()
                        .filter(|key| key.extension().and_then(OsStr::to_str) == Some("md"))
                        .collect();

                    for path in md {
                        posts.build_post(path)?;
                    }
                    break;
                }
                Some("md") => posts.build_post(file)?,
                _ => (),
            }
        }

        //If a post is updated, the metadata could also be updated.
        //So the list of posts will also need to be updated.
        //Updating templates has the same requirement.
        if !outdated_files.is_empty() {
            match posts.build_list() {
                Ok(_) => info!("Sucessfully built posts."),
                Err(err) => warn!("Failed to compile posts\n{err}"),
            };
        }

        std::thread::sleep(POLL_DURATION);
    }
}
