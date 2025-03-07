use comrak::{ComrakOptions, markdown_to_html};
use rocket::State;
use rocket::form::Form;
use rocket::fs::{FileServer, relative};
use rocket::{response::content::RawHtml, tokio::fs};
use std::path::Path;
use std::sync::Mutex;
use tera::{Context, Tera};

const PASSWORD: &str = "SUPERSECRETPASSWORD1234";

#[macro_use]
extern crate rocket;

struct Posts {
    posts: Vec<String>,
}

#[get("/")]
async fn index(posts: &State<Posts>) -> RawHtml<String> {
    let tera = Tera::new("web/*").unwrap();

    let mut context = Context::new();
    context.insert("posts", &posts.posts);

    let rendered = tera.render("index.html", &context).unwrap();
    RawHtml(rendered)
}

#[derive(FromForm)]
struct PostData {
    pass: String,
    name: String,
    content: String,
}

#[post("/posts/create", data = "<post_data>")]
async fn create_post(post_data: Form<PostData>, posts: &State<Mutex<Posts>>) -> String {
    if post_data.pass == PASSWORD {
        match fs::write(
            ("web/posts/".to_owned() + &post_data.name.clone().to_owned() + ".html").as_str(),
            "<head><link rel=\"stylesheet\" href=\"public/styles.css\"></head>".to_string()
                + &markdown_to_html(&post_data.content.clone(), &ComrakOptions::default()),
        )
        .await
        {
            Ok(_) => {
                let mut posts = posts.lock().unwrap();
                posts.posts.push(post_data.name.clone());
                "Post created".to_string()
            }
            Err(_) => "Error creating post".to_string(),
        }
    } else {
        "Invalid password".to_string()
    }
}
#[get("/posts/<file>")]
async fn files(file: &str) -> Result<RawHtml<String>, RawHtml<String>> {
    let path = format!("web/posts/{}", file);
    if !Path::new(&path).exists() {
        return Err(RawHtml("Post not found".to_string()));
    }
    let contents = fs::read_to_string(path).await.unwrap();
    Ok(RawHtml(contents))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, files, create_post])
        .mount("/public", FileServer::from(relative!("web/public")))
        .manage(Posts { posts: vec![] })
}
