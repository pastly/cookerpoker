use super::*;

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![index_logged_in, index,]
}

#[get("/", rank = 2)]
fn index() -> Redirect {
    Redirect::to("/index.html")
}

#[get("/")]
fn index_logged_in(u: User) -> Template {
    let mut c = Context::new();
    c.insert("account", &u.0);
    Template::render("logged_in", &c.into_json())
}
