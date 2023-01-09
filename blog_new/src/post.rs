use crate::{build_path, metadata, warn, Metadata};
use pulldown_cmark::*;
use std::{
    error::Error,
    fs::{self},
    path::Path,
};

pub fn build(post_template: &str, markdown: &Path) -> Result<(), Box<dyn Error>> {
    //Read the markdown file.
    let file = fs::read_to_string(markdown)?;
    let (file, metadata) = if file.contains("~~~") {
        let end = file[2..].find("~~~").ok_or("Corrupt metadata.")?;
        (
            file[end + "~~~".len()..].trim_start(),
            metadata(&file, markdown)?,
        )
    } else {
        warn!("{:?} missing metadata.", markdown);
        (file.as_str(), Metadata::default())
    };

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

    let path = build_path(markdown);

    //Write the post to disk.
    fs::write(path, post)?;

    Ok(())
}
