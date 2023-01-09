mod post_list;

use chrono::{DateTime, Datelike, Utc};
use log::{info, warn};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, Cursor},
    path::{Path, PathBuf},
    time::Duration,
};

const MARKDOWN_PATH: &str = "markdown";
const BUILD_PATH: &str = "build";
const POLL_DURATION: Duration = Duration::from_millis(500);

fn update_files(files: &mut HashMap<PathBuf, String>) -> Vec<PathBuf> {
    let mut outdated_files = Vec::new();

    walkdir::WalkDir::new(MARKDOWN_PATH)
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
        .for_each(|path| {
            let hash = hash(&path).unwrap_or_default();
            if let Some(old_hash) = files.get(&path) {
                if &hash != old_hash {
                    outdated_files.push(path.clone());
                }
            }

            if files.insert(path.clone(), hash).is_none() {
                outdated_files.push(path);
            };
        });

    outdated_files
}

fn run() {
    info!("Watching files in {:?}", Path::new(MARKDOWN_PATH));
    let mut files = HashMap::new();
    loop {
        std::thread::sleep(POLL_DURATION);
        let outdated_files = update_files(&mut files);
        if !outdated_files.is_empty() {
            //TODO: Build the posts page with all the post metadata.
            let metadata: Vec<Metadata> = files.keys().flat_map(metadata).collect();
            post_list::build(&metadata);

            for file in outdated_files {
                match build_markdown(&file) {
                    Ok(_) => info!("Re-compiled: {file:?}"),
                    Err(err) => warn!("Failed to compile: {file:?}\n{err}"),
                }
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct Metadata {
    pub title: String,
    pub summary: String,
    pub date: String,
}

fn metadata(path: impl AsRef<Path>) -> Result<Metadata, Box<dyn Error>> {
    let file = fs::read_to_string(&path)?;

    let pattern = "~~~\n";
    let start = file.find(pattern).ok_or("")?;
    let end = file[start + pattern.len()..].find(pattern).ok_or("")?;

    //Ignore the last newline.
    let config = &file[start + pattern.len()..end + pattern.len() - 1];

    let mut metadata = Metadata::default();

    for line in config.split('\n') {
        let (k, v) = line.split_once(':').ok_or("")?;
        match k {
            "title" => metadata.title = v.to_string(),
            "summary" => metadata.summary = v.to_string(),
            _ => unreachable!(),
        }
    }

    let date = fs::metadata(path)?.created()?;
    let now: DateTime<Utc> = date.into();
    let month = match now.month() {
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

    metadata.date = format!("{} {} {}", now.day(), month, now.year());

    Ok(metadata)
}

fn build_markdown(path: &Path) -> io::Result<()> {
    use pulldown_cmark::*;

    //Read the markdown file.
    let string = fs::read_to_string(path)?;
    let markdown = &string[3..];
    let end = markdown.find("~~~\n").unwrap();
    let markdown = &markdown[end + "~~~\n".len()..];

    //Convert the markdown to html.
    let parser = Parser::new_ext(markdown, Options::all());
    let mut html = String::new();
    html::push_html(&mut html, parser);

    //Convert "markdown/test.md" to "build/test.html"
    let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
    name.pop();
    name.pop();
    name.push_str("html");

    let path = PathBuf::from(BUILD_PATH).join(name);

    //Save the compiled template.
    fs::write(path, html)?;

    Ok(())
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

fn build_all() {
    info!("Compliling files in {:?}", Path::new(MARKDOWN_PATH));
    walkdir::WalkDir::new(MARKDOWN_PATH)
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
        .for_each(|path| match build_markdown(&path) {
            Ok(_) => info!("Sucessfully compiled: {path:?}"),
            Err(_) => warn!("Failed to compile: {path:?}"),
        });
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
    simple_logger::SimpleLogger::new().init().unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();

    if let Some(arg) = args.get(0) {
        match arg.as_str() {
            "build" => build_all(),
            "clean" => clean(),
            "run" => run(),
            _ => help(),
        }
    } else {
        run();
    }
}
