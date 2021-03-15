use pulldown_cmark::{html, Parser};

fn main() {
    let markdown_str = r#"# Hello
人間は愚かな生物。

[俺のブログ](https://blog.himanoa.net)
"#;
    let parser = Parser::new(markdown_str);
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    println!("{}", html_buf);
}
