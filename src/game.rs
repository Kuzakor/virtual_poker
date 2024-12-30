use crate::database;
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};

/* look game
    - renders the game_looking template
*/
#[get("/lookgame")]
pub fn look_game() -> Template {
    Template::render("game_looking", context! {})
}

/* check game
    - cookies: to get user cookie
    - this site is pinged every 1 second by javascript in game_looking
    - returns a String for the javascript to interpret
    - "none": if no cookie
    - "/<game rowid in sql database>/<host or guest>": connected game and basic info of it
    - "created a ticket": if a new game ticket have been created
*/
#[get("/check-game")]
pub fn check_game(cookies: &CookieJar<'_>) -> String {
    let database = database::Database::connect();
    let cookie = match cookies.get("user") {
        Some(cookie) => cookie.value(),
        None => return String::from("none"),
    };
    match database.join_open_game(&cookie) {
        Ok(rowid) => rowid,
        Err(_) => {
            let _ = database.new_game(&cookie);
            String::from("created a ticket")
        }
    }
}

/* game
    - rowid: the row id in sql database
    - typ: either host or guest, info on user connection type
    - gets the opponent cookie using the rowid and typ
    - gets the user and renders the info
*/
#[get("/game/<rowid>/<typ>")]
pub fn game(rowid: u64, typ: &str) -> Template {
    let database = database::Database::connect();
    let opponent_cookie = database.get_opponent_cookie(typ, rowid).unwrap();
    let opponent = database.get_user(&*opponent_cookie, "cookie").unwrap();

    Template::render(
        "game",
        context! {
            pfp: opponent.profile_picture,
            username: opponent.username,
            money: opponent.money,
        },
    )
}
