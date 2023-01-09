use crate::{metadata, BUILD_PATH};
use pulldown_cmark::*;
use std::{
    error::Error,
    fs::{self},
    path::{Path, PathBuf},
};

lazy_static::lazy_static! {
    static ref TEMPLATE: String =
        String::from_utf8_lossy(include_bytes!("../templates/post.html")).to_string();
}

pub fn build(markdown: &Path) -> Result<(), Box<dyn Error>> {
    //Read the markdown file.
    let string = fs::read_to_string(markdown)?;
    let string = &string[3..];
    let end = string.find("~~~\n").unwrap();
    let string = &string[end + "~~~\n".len()..];

    //Convert the markdown to html.
    let parser = Parser::new_ext(string, Options::all());
    let mut html = String::new();
    html::push_html(&mut html, parser);

    //Get the metadata from the markdown file.
    let metadata = metadata(markdown)?;
    //Generate the post using the metadata and html.
    let post = TEMPLATE
        .replace("<!-- title -->", &metadata.title)
        .replace("<!-- date -->", &metadata.date)
        .replace("<!-- content -->", &html);

    //Convert "markdown/example.md" to "build/example.html"
    let mut name = markdown.file_name().unwrap().to_str().unwrap().to_string();
    name.pop();
    name.pop();
    name.push_str("html");
    let path = PathBuf::from(BUILD_PATH).join(name);

    //Write the post to disk.
    fs::write(path, post)?;

    Ok(())
}
