use frontmatter::{parse, Yaml};
use pulldown_cmark::{html, Parser};
use std::io::{BufRead, BufReader};
use std::{fs, io::Read, io::Write};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
use tera::{Context, Tera};

#[derive(Debug)]
struct PostMeta {
    path: String,
}

fn parse_frontmatter(s: &str) -> PostMeta {
    let front = parse(&s);
    match front {
        Ok(s) => {
            match s {
                Some(json) => {
                    let path = &json["path"];
                    match path {
                        Yaml::String(a) => PostMeta {
                            path: a.to_string(),
                        },
                        _ => PostMeta {
                            path: "".to_string(),
                        },
                    }
                }
                // TODO: should raise exception
                None => PostMeta {
                    path: "".to_string(),
                },
            }
        }
        // TODO: should raise exception
        Err(_) => PostMeta {
            path: "".to_string(),
        },
    }
}

fn delete_frontmatter(f: &File) -> String {
    let mut res = "".to_string();
    for (idx, line) in BufReader::new(f).lines().enumerate() {
        if (idx > 9) {
            let line = line.unwrap();
            res = res.clone() + line.as_str();
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

    // 実行位置からの相対パス
    let dir = fs::read_dir("./src/contents");

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
                println!("{:?}", front);

                // 2回 file open せなあかんのはどうにかしたい
                f = File::open(a_p).unwrap();
                let res = delete_frontmatter(&f);
                let res_str = res.as_str();
                println!("res_str{:?}", res_str);

                // TODO: frontmatter 部分の削除
                let parser = Parser::new(res_str);
                let mut html_buf = String::new();
                html::push_html(&mut html_buf, parser);
                context.insert("content", &html_buf);

                let rendered = tera.render("post.html", &context);
                match rendered {
                    Ok(render) => {
                        let filename = format!("public/{}.html", front.path.as_str());
                        let mut file = fs::File::create(filename).unwrap();
                        file.write_all(render.as_bytes()).unwrap();
                    }
                    Err(why) => {
                        println!("{:?}", why)
                    }
                }
            }
        }
    }
}
