use crate::Metadata;
use std::fs;

pub fn build(list_template: &str, list_item_template: &str, metadata: &[Metadata]) {
    let index = list_template.find("<!-- posts -->").unwrap();
    let mut template = list_template.replace("<!-- posts -->", "");

    for post in metadata.iter().rev() {
        let (day, month, year) = post.date();
        let list_item = list_item_template
            .replace("~link~", &post.path)
            .replace("<!-- title -->", &post.title)
            .replace("<!-- date -->", &format!("{day} {month} {year}"))
            .replace("<!-- read_time -->", &post.read_time())
            .replace("<!-- word_count -->", &post.word_count.to_string())
            .replace("<!-- summary -->", &post.summary);

        template.insert_str(index, &list_item);
    }

    fs::write("build/post_list.html", template).unwrap();
}
