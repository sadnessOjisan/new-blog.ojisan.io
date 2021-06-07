use frontmatter::{parse, Yaml};
use pulldown_cmark::{html, Event, LinkType, Parser, Tag};
use serde::Serialize;
use std::borrow::Cow::Owned;
use std::io::{BufRead, BufReader};
use std::{fs, io::Read, io::Write};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
use tera::{Context, Tera};

mod file_system;

#[derive(Debug)]
struct PostMeta {
    path: String,
    title: String,
    tags: Vec<String>,
    created_at: String,
}

#[derive(Serialize, Debug)]
struct IndexItem {
    title: String,
    path: String,
    created_at: String,
}

fn parse_frontmatter(s: &str) -> PostMeta {
    let front = parse(&s);
    let yaml = front.ok().unwrap().unwrap();
    let path = &yaml["path"];
    let title = &yaml["title"];
    let tags = &yaml["tags"];
    let created_at = &yaml["created"];

    PostMeta {
        path: path.as_str().unwrap().to_string(),
        title: title.as_str().unwrap().to_string(),
        tags: tags
            .as_vec()
            .unwrap()
            .into_iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect(),
        created_at: created_at.as_str().unwrap().to_string(),
    }
}

fn delete_frontmatter(f: &File) -> String {
    let FRONTMATTER_LINES = 9;
    let mut res = "".to_string();
    for (idx, line) in BufReader::new(f).lines().enumerate() {
        if (idx > FRONTMATTER_LINES) {
            let line = line.unwrap();
            res = res.clone() + line.as_str() + "\n";
        }
    }
    res
}

fn main() {
    let tera = match Tera::new("src/templates/*.html") {
        Ok(mut t) => {
            t.autoescape_on(vec![]); // html そのものを埋め込みたいから escape しない。
            t
        }
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let mut context = Context::new();
    let mut topContext = Context::new();

    // 実行位置からの相対パス
    let dir = fs::read_dir("./src/contents");

    let mut items: Vec<IndexItem> = vec![];

    // https://doc.rust-jp.rs/rust-by-example-ja/std_misc/fs.html
    match dir {
        Err(why) => println!("! {:?}", why.kind()),
        Ok(paths) => {
            for path in paths {
                let p = path.unwrap().path();
                let s = p.to_str().unwrap();
                let article_path = s.to_string() + "/index.md";
                let a_p = Path::new(&article_path);
                let mut f = File::open(a_p).unwrap();
                let mut s = String::new();
                f.read_to_string(&mut s);
                let front = parse_frontmatter(&s);

                // 2回 file open せなあかんのはどうにかしたい
                f = File::open(a_p).unwrap();
                let res = delete_frontmatter(&f);

                let mut description = "".to_string();
                let mut cnt = 0;
                let parser = Parser::new(&res).map(|event| match event.clone() {
                    Event::Text(text) => {
                        let description_parts = text.into_string();
                        let description_parts_len = description_parts.len();
                        if(cnt < 100){
                            cnt = cnt + description_parts_len;
                            let description_parts_str = description_parts.as_str();
                            description = format!("{}{}",description , description_parts_str);
                        }
                        event
                    },
                    _ => event
                });
                let mut html_buf = String::new();
                html::push_html(&mut html_buf, parser);
                context.insert("description", &description);
                context.insert("content", &html_buf);
                context.insert("description", &description);
                context.insert("title", &front.title);
                context.insert("path", &front.path);
                context.insert("created_at", &front.created_at);
                let dir = fs::read_dir("./public");
                let target = format!("./public/{}", front.path.as_str());
                let target_path = Path::new(target.as_str());
                file_system::copy(p, target_path);
                let rendered = tera.render("post.html", &context);
                let item = IndexItem {
                    title: front.title,
                    path: front.path,
                    created_at: front.created_at,
                };
                items.push(item);
                match rendered {
                    Ok(render) => {
                        let filename = format!("{}/index.html", target);
                        let mut file = fs::File::create(filename).unwrap();
                        file.write_all(render.as_bytes()).unwrap();
                    }
                    Err(why) => {
                        println!("{:?}", why)
                    }
                }
            }
            items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            topContext.insert("items", &items);
            let top_rendered = tera.render("index.html", &topContext);
            match top_rendered {
                Ok(render) => {
                    let target = "./public";
                    let filename = format!("{}/index.html", target);
                    let mut file = fs::File::create(filename).unwrap();
                    file.write_all(render.as_bytes()).unwrap();
                }
                Err(why) => {
                    println!("top_rendered error -> {:?}", why)
                }
            }
        }
    }

    fs::copy("src/style/post.css", "public/post.css");
    fs::copy("src/style/top.css", "public/top.css");
    fs::copy("src/style/reset.css", "public/reset.css");
    fs::copy("src/sw/manifest.json", "public/manifest.json");
    fs::copy("src/sw/sw.js", "public/sw.js");
    fs::copy("src/assets/favicon.ico", "public/favicon.ico");
}
