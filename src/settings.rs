use crate::database;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};

/* change data form
    - r#data: new data
*/
#[derive(FromForm)]
struct ChangeData<'r> {
    r#data: &'r str,
}

/* get - change website
    - datatype: username, password etc.
    - renders a dedicated website depending on the changed data
*/
#[get("/change/<datatype>")]
pub fn change(datatype: String) -> Template {
    let (typ, special) = match datatype.as_str() {
        "password" => ("password", "change"),
        "picture" => ("file", "file"),
        _ => ("text", "change"),
    };
    Template::render(
        "change",
        context! {special: special, typ: typ, name: datatype},
    )
}

/* post - profile picture update, asynchronous
    - file: file input
    - cookies: to know for whom the pfp is
    - reads the user from the database based on cookie
    - copies the file to /static/pictures
    - updates the database
    - redirects to /
*/
#[post("/file/picture", data = "<file>")]
pub async fn change_file_post(
    mut file: Form<TempFile<'_>>,
    cookies: &CookieJar<'_>,
) -> std::io::Result<Redirect> {
    let database = database::Database::connect();
    let cookie = cookies.get("user");
    if cookie.is_none() {
        return Ok(Redirect::to("/"));
    }

    let path = format!("static/pictures/{}", file.name().unwrap());
    let _ = file.copy_to(&path).await?;

    database.update_user(path.as_str(), "profilePicture", cookie.unwrap().value());
    Ok(Redirect::to("/"))
}

/* post - profile other info update
    - change: struct input
    - cookies: to know for whom the data is
    - reads the user from the database based on cookie
    - updates the database
    - redirects to /
*/
#[post("/change/<datatype>", data = "<change>")]
pub fn change_post(datatype: &str, change: Form<ChangeData>, cookies: &CookieJar<'_>) -> Redirect {
    let database = database::Database::connect();
    let cookie = cookies.get("user");
    if cookie.is_none() {
        return Redirect::to("/");
    }
    database.update_user(change.data, datatype, cookie.unwrap().value());
    Redirect::to("/")
}
