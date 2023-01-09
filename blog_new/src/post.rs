use crate::{build_path, metadata};
use pulldown_cmark::*;
use std::{
    error::Error,
    fs::{self},
    path::Path,
};

pub fn build(post_template: &str, markdown: &Path) -> Result<(), Box<dyn Error>> {
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
    let post = post_template
        .replace("<!-- title -->", &metadata.title)
        .replace("<!-- date -->", &metadata.date)
        .replace("<!-- content -->", &html);

    let path = build_path(markdown);

    //Write the post to disk.
    fs::write(path, post)?;

    Ok(())
}
