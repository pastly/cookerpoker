use super::*;

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![index, index_anon,]
}

#[get("/")]
fn index(u: User) -> Template {
    let mut c = Context::new();
    c.insert("account", &(*u));
    Template::render("index", &c.into_json())
}
#[get("/", rank = 2)]
fn index_anon() -> Template {
    let c = Context::new();
    Template::render("index", &c.into_json())
}
//#[get("/", rank = 2)]
//fn index() -> Redirect {
//    Redirect::to("/index.html")
//}
//
//#[get("/")]
//fn index_logged_in(u: User) -> Template {
//    let mut c = Context::new();
//    c.insert("account", &(*u));
//    Template::render("logged_in", &c.into_json())
//}
