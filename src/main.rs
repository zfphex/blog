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
const BUILD_PATH: &str = "site";
const POLL_DURATION: Duration = Duration::from_millis(120);
const INDEX: &str = "site\\index.html";
const POST: &str = "templates\\post.html";
const LIST: &str = "templates\\post_list.html";
const LIST_ITEM: &str = "templates\\post_list_item.html";
const CSS: &str = "site\\assets\\style.css";
const CSS_MIN: &str = "site\\assets\\style-min.css";

mod hex;
mod html;
mod syntax_highlighting;

fn now() -> String {
    Local::now().time().format("%H:%M:%S").to_string()
}

fn minify(html: &str) -> Vec<u8> {
    let mut cfg = minify_html::Cfg::spec_compliant();
    cfg.keep_closing_tags = true;
    cfg.keep_html_and_head_opening_tags = true;
    cfg.minify_css = true;
    cfg.minify_js = true;
    minify_html::minify(html.as_bytes(), &cfg)
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        print!("\x1b[90m{} \x1b[94mINFO\x1b[0m {}:{}:{} ", now(), file!(), line!(), column!());
        println!($($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        print!("\x1b[90m{} \x1b[93mWARN\x1b[0m {}:{}:{} ", now(), file!(), line!(), column!());
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

    //TOOD: Maybe don't use a string, it's kinda slow dude.
    while len > 0 {
        output.fill(&mut block);
        let hex_str = hex::encode(&block[..]);
        let take_bytes = std::cmp::min(len, block.len() as u64);
        hex.push_str(&hex_str[..2 * take_bytes as usize]);
        len -= take_bytes;
    }

    Ok(hex)
}

struct List {
    pub posts: HashMap<PathBuf, Post>,
}

impl List {
    pub fn new() -> Self {
        Self {
            posts: HashMap::new(),
        }
    }
    pub fn update(&mut self, templates: &mut Templates) -> Result<(), Box<dyn Error>> {
        let files: Vec<PathBuf> = fs::read_dir(MARKDOWN_PATH)?
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
        for file in files {
            //Generate a hash for the file.
            let hash = hash(&file)?;

            if let Some(post) = self.posts.get_mut(&file) {
                //File is out of date.
                if hash != post.hash {
                    rebuild_list = true;
                    match Post::new(&post_template, &file, hash) {
                        Ok(p) => *post = p,
                        Err(err) => warn!("Failed to compile: {file:?}\n{err}"),
                    };
                }
            } else {
                //File is new.
                rebuild_list = true;
                match Post::new(&post_template, &file, hash) {
                    Ok(post) => {
                        self.posts.insert(file, post);
                    }
                    Err(err) => warn!("Failed to compile: {file:?}\n{err}"),
                };
            }
        }

        if rebuild_list {
            self.build_list(templates)?;
        }

        Ok(())
    }
    pub fn build_list(&self, templates: &mut Templates) -> Result<(), Box<dyn Error>> {
        let (list_template, _) = &templates.list;
        let (list_item_template, _) = &templates.list_item;

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

#[derive(Debug)]
pub struct Post {
    pub metadata: Metadata,
    pub build_path: PathBuf,
    pub hash: String,
}

impl Post {
    pub fn new(post_template: &str, path: &Path, hash: String) -> Result<Self, Box<dyn Error>> {
        use pulldown_cmark::*;

        //Read the markdown file.
        let file = fs::read_to_string(path)?;
        let metadata = Metadata::new(&file, path)?;
        let file = &file[metadata.end_position..].trim_start();

        //Convert the markdown to html.
        let parser = Parser::new_ext(file, Options::all());
        let mut html = String::new();
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
        let build_path = PathBuf::from(BUILD_PATH).join(name);

        let minified_post = minify(&post);
        fs::write(&build_path, minified_post)?;
        info!("Created new post: {path:?}");

        Ok(Self {
            metadata,
            build_path,
            hash,
        })
    }
}

#[derive(Default)]
struct Templates {
    pub post: (String, String),
    pub list: (String, String),
    pub list_item: (String, String),
}

impl Templates {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            post: (fs::read_to_string(POST)?, hash(POST)?),
            list: (fs::read_to_string(LIST)?, hash(LIST)?),
            list_item: (fs::read_to_string(LIST_ITEM)?, hash(LIST_ITEM)?),
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    //Make sure the build folder exists.
    let _ = fs::create_dir(BUILD_PATH);

    info!("Watching files in {:?}", Path::new(MARKDOWN_PATH));

    let mut t = Templates::new()?;
    let mut posts = List::new();
    let css = hash(CSS)?;

    loop {
        //Make sure the templates and `style-min.css` are up to date.
        let new_hash = hash(CSS)?;
        if new_hash != css {
            let css = fs::read_to_string(CSS)?;
            let min = minify(&css);
            fs::write(&CSS_MIN, min)?;
        }

        let new_hash = hash(POST)?;
        if new_hash != t.post.1 {
            info!("Compiled: {:?}", POST);
            t.post.0 = fs::read_to_string(POST)?;
            t.post.1 = new_hash;

            //Re-build the posts.
            //Delete the old hashes and List::update() will do the work.
            for (_, v) in posts.posts.iter_mut() {
                v.hash = String::new();
            }
        }

        let mut rebuild_list = false;

        let new_hash = hash(LIST)?;
        if new_hash != t.list.1 {
            info!("New: {} Old: {}", new_hash, t.post.1);
            info!("Compiled: {:?}", LIST);
            t.list.0 = fs::read_to_string(LIST)?;
            t.list.1 = new_hash;
            rebuild_list = true;
        }

        let new_hash = hash(LIST_ITEM)?;
        if new_hash != t.list_item.1 {
            info!("Compiled: {:?}", LIST_ITEM);
            t.list_item.0 = fs::read_to_string(LIST_ITEM)?;
            t.list_item.1 = new_hash;
            rebuild_list = true;
        }

        if rebuild_list {
            posts.build_list(&mut t)?;
        }

        posts.update(&mut t)?;

        std::thread::sleep(POLL_DURATION);
    }
}
