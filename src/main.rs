mod database;
mod login;
mod register;
mod settings;

#[macro_use]
extern crate rocket;
use crate::database::Database;
use rocket::fs::FileServer;
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};

/* index site
    - cookies: user cookie
    - if logged in: renders user info
    - if logged out: renders main site with login/register buttons
*/
#[get("/")]
fn index(cookies: &CookieJar<'_>) -> Template {
    let database = Database::connect();
    let user = match cookies.get("user") {
        Some(cookie) => database.get_user(cookie.value(), "cookie"),
        None => return Template::render("index", context! {}),
    };
    let user = user.unwrap();
    Template::render(
        "user",
        context! {
            pfp: user.profile_picture,
            username: user.username,
            money: user.money,
        },
    )
}

/*launch
    - mounts static as /static
*/
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                login::login,
                register::register,
                register::register_post,
                login::login_post,
                login::logout,
                settings::change,
                settings::change_post,
                settings::change_file_post
            ],
        )
        .mount("/static", FileServer::from("static"))
        .attach(Template::fairing())
}
