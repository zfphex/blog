use crate::Metadata;
use std::fs;

lazy_static::lazy_static! {
    static ref TEMPLATE: String =
        String::from_utf8_lossy(include_bytes!("../templates/post_list.html")).to_string();
    static ref LIST_ITEM: String =
        String::from_utf8_lossy(include_bytes!("../templates/post_list_item.html")).to_string();
}

pub fn build(metadata: &[Metadata]) {
    let index = TEMPLATE.find("<!-- posts -->").unwrap();
    let mut template = TEMPLATE.replace("<!-- posts -->", "");

    for post in metadata.iter().rev() {
        let list_item = LIST_ITEM
            .replace("<!-- title -->", &post.title)
            .replace("<!-- user -->", "User")
            .replace("<!-- date -->", &post.date)
            .replace("<!-- read_time -->", "Read Time")
            .replace("<!-- word_count -->", "Word Count")
            .replace("<!-- summary -->", &post.summary);

        template.insert_str(index, &list_item);
    }

    fs::write("build/post_list.html", template).unwrap();
}
