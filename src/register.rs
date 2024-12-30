use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};
use crate::database;

/* register form
    - r#username: username
    - r#password: password
*/
#[derive(FromForm)]
struct RegisterForm<'r> {
    r#username: &'r str,
    r#password: &'r str,
}

/* get - register website
    - message: if the site is re-rendered because of already-used username
    - renders the form with or without the message
*/
#[get("/register/<message>")]
pub fn register(message: bool) -> Template {
    let m = match message {
        true => "Username already takem",
        false => " ",
    };
    Template::render("register", context! {message: m})
}

/* post - register website
    - form: register form
    - cookies: to automatically log in the new user
    - connects to the database
    - checks if the username is already taken: redirects to get website with a message if true
    - creates the new user in the database
    - sets the cookie: logins the user
    - redirects to /
*/
#[post("/register", data = "<form>")]
pub fn register_post(form: Form<RegisterForm<'_>>, cookies: &CookieJar<'_>) -> Redirect{
    let data = database::Database::connect();
    if data.get_user(form.username, "username").is_err() {
        let _ = data.new_user(form.username, form.password);
        let user = data.get_user(form.username, "username").unwrap();
        cookies.add(Cookie::new("user", user.cookie));
        return Redirect::to("/");
    }
    Redirect::to("/register/true")
}