use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};
use crate::database;
use crate::database::Database;

/* login form
    - r#username: username
    - r#password: password
*/
#[derive(FromForm)]
struct LoginForm<'r> {
    r#username: &'r str,
    r#password: &'r str,
}

/* get - login website
    - message: optional message for the user
    - renders the form with or without the message
*/
#[get("/login/<message>")]
pub fn login(message: &str) -> Template {
    let m = match message{
        "Password" => "Wrong password",
        "User" => "No such user",
        _ => " "
    };
    Template::render("login", context! {message: m})
}

/* post - login website
    - form: login form
    - cookies: to log in the user
    - connects to the database
    - checks if the given user exists: if not, redirects to /login with a message
    - checks if the password is correct: if not redirects to /login with a message
    - sets the cookie: logins
    - redirects to /
*/
#[post("/login", data = "<form>")]
pub fn login_post(form: Form<LoginForm<'_>>, cookies: &CookieJar<'_>) -> Redirect {
    let database = Database::connect();
    let user = database.get_user(form.username, "username");
    match user {
        Some(user) => {
            if database::password_check(&user.password_hash, &form.password) {
                cookies.add(Cookie::new("user", user.cookie));
            } else {
                return Redirect::to("/login/Password");
            }
        }
        None => return Redirect::to("/login/User")
    }
    Redirect::to("/")
}

/* logout
    - cookies: to be able to modify them
    - removes the user cookie: logs him out
*/
#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::from("user"));
    Redirect::to("/")
}
