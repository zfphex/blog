use chrono::{DateTime, Datelike, Utc};
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fs::{self, File},
    io::Cursor,
    path::{Path, PathBuf},
    time::Duration,
};
use walkdir::WalkDir;

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        print!("\x1b[94mINFO\x1b[0m ");
        println!($($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        print!("\x1b[93mWARN\x1b[0m '{}:{}:{}' ", file!(), line!(), column!());
        println!($($arg)*);
    }};
}

const MARKDOWN_PATH: &str = "markdown";
const TEMPLATE_PATH: &str = "templates";
const BUILD_PATH: &str = "build";
const POLL_DURATION: Duration = Duration::from_millis(500);

struct Watcher {
    pub files: HashMap<PathBuf, String>,
}

impl Watcher {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }
    pub fn update(&mut self) -> Vec<PathBuf> {
        WalkDir::new(MARKDOWN_PATH)
            .max_depth(1)
            .into_iter()
            .chain(WalkDir::new(TEMPLATE_PATH).into_iter())
            .flatten()
            .map(|dir_entry| dir_entry.path().to_path_buf())
            .filter(|path| {
                if let Some(ex) = path.extension() {
                    let ex = ex.to_ascii_lowercase();
                    ex == "md" || ex == "html"
                } else {
                    false
                }
            })
            .filter_map(|path| {
                //Generate a hash for the file.
                let hash = hash(&path).unwrap_or_default();

                //Check if the hashes match.
                match self.files.insert(path.clone(), hash.clone()) {
                    Some(old_hash) => {
                        if hash != old_hash {
                            return Some(path);
                        }
                    }
                    None => return Some(path),
                }

                None
            })
            .collect()
    }
    pub fn md(&self) -> Vec<&PathBuf> {
        self.files
            .keys()
            .filter(|key| key.extension().and_then(OsStr::to_str) == Some("md"))
            .collect()
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    info!("Watching files in {:?}", Path::new(MARKDOWN_PATH));
    let mut watcher = Watcher::new();
    let mut posts = Posts::new();

    loop {
        let outdated_files = watcher.update();
        let empty = outdated_files.is_empty();
        let mut updated = false;

        for file in outdated_files {
            match file.extension().and_then(OsStr::to_str) {
                Some("md") => match Post::new(&posts.post_template, &file) {
                    Ok(post) => {
                        info!("Compiled: {file:?}");
                        post.write()?;
                        posts.insert(file, post);
                    }
                    Err(err) => warn!("Failed to compile: {file:?}\n{err}"),
                },
                Some("html") if !updated => {
                    updated = true;
                    posts.update_templates();

                    for path in watcher.md() {
                        match Post::new(&posts.post_template, path) {
                            Ok(post) => {
                                info!("Compiled: {path:?}");
                                post.write()?;
                                posts.insert(path.clone(), post);
                            }
                            Err(err) => warn!("Failed to compile: {path:?}\n{err}"),
                        }
                    }
                }
                _ => (),
            }
        }

        //If a post is updated, the metadata could also be updated.
        //So the list of posts will also need to be updated.
        //Updating templates has the same requirement.
        if !empty {
            posts.build();
        }

        std::thread::sleep(POLL_DURATION);
    }
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
    pub fn insert(&mut self, path: PathBuf, post: Post) {
        self.posts.insert(path, post);
    }
    pub fn update_templates(&mut self) {
        info!("Re-building templates.");
        self.list_template = fs::read_to_string("templates/post_list.html").unwrap();
        self.list_item_template = fs::read_to_string("templates/post_list_item.html").unwrap();
        self.post_template = fs::read_to_string("templates/post.html").unwrap();
    }
    ///Build the list of posts.
    pub fn build(&mut self) {
        info!("Compiled: \"build\\\\post_list.html\"");
        let index = self.list_template.find("<!-- posts -->").unwrap();
        let mut template = self.list_template.replace("<!-- posts -->", "");

        for post in self.posts.values() {
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

        fs::write("build/post_list.html", template).unwrap();
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub summary: String,
    pub date: DateTime<Utc>,
    pub link_path: String,
    pub real_path: PathBuf,
    pub word_count: usize,
    pub read_time: f32,

    pub end_position: usize,
}

impl Metadata {
    pub fn new(file: &str, path: &Path) -> Self {
        let config = file.get(3..).unwrap().trim_start();

        let mut title = String::new();
        let mut summary = String::new();
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
                        _ => continue,
                    }
                }
            }
        }

        let mut pathbuf = path.to_path_buf();
        pathbuf.set_extension("html");

        //Rough estimate of the word count. Doesn't actually count alphanumerically.
        let word_count = file[end..].split(|c: char| c.is_whitespace()).count();

        Metadata {
            title,
            summary,
            date: fs::metadata(path).unwrap().created().unwrap().into(),
            link_path: pathbuf.file_name().unwrap().to_str().unwrap().to_string(),
            real_path: path.to_path_buf(),
            read_time: word_count as f32 / 250.0,
            word_count,
            end_position: end,
        }
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

        let metadata = Metadata::new(&file, path);
        let file = &file[metadata.end_position..].trim_start();

        //Convert the markdown to html.
        let parser = Parser::new_ext(file, Options::all());
        let mut html = String::new();
        html::push_html(&mut html, parser);

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
        fs::write(&self.build_path, &self.html)
    }
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

fn clean() {
    let markdown_files: Vec<PathBuf> = walkdir::WalkDir::new(MARKDOWN_PATH)
        .into_iter()
        .flatten()
        .map(|dir_entry| dir_entry.path().to_path_buf())
        .filter(|path| {
            if let Some(ex) = path.extension() {
                ex.to_ascii_lowercase() == "md"
            } else {
                false
            }
        })
        .collect();

    let build_files: Vec<PathBuf> = walkdir::WalkDir::new(BUILD_PATH)
        .into_iter()
        .flatten()
        .map(|dir_entry| dir_entry.path().to_path_buf())
        .filter(|path| path.is_file())
        .collect();

    let expected_files: Vec<PathBuf> = markdown_files
        .into_iter()
        .map(|file| {
            let mut name = file.file_name().unwrap().to_str().unwrap().to_string();
            name.pop();
            name.pop();
            name.push_str("html");
            PathBuf::from(BUILD_PATH).join(name)
        })
        .collect();

    for file in build_files {
        if !expected_files.contains(&file) {
            match fs::remove_file(&file) {
                Ok(_) => info!("Removed unexpected file: {file:?}"),
                Err(_) => warn!("Failed to remove unexpected file: {file:?}"),
            }
        }
    }
}

fn help() {
    println!(
        r#"Usage
   md2html [<command> <args>]

Options
   run           Watch for file changes and compile.
   build         Compile all markdown files.
   clean         Remove unused files."#
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if let Some(arg) = args.get(0) {
        match arg.as_str() {
            "build" => todo!(),
            "clean" => clean(),
            "run" => run().unwrap(),
            _ => help(),
        }
    } else {
        run().unwrap();
    }
}
