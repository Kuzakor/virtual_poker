use pwhash::bcrypt;
use sqlite;

/* database
    - connection with sqlite database
*/
pub struct Database {
    connection: sqlite::Connection,
}

/* user
    - conversion of database columns into an object
*/
#[derive(Debug)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub cookie: String,
    pub profile_picture: String,
    pub money: i64,
}

/*pub struct PendingGame {
    pub host: User,
    pub guest: Option<User>
}*/

impl Database {
    /* connect
        - creates a new connection to the database
    */
    pub fn connect() -> Database {
        Self {
            connection: sqlite::open("data/database").unwrap(),
        }
    }

    /* new user
        - username: username
        - password: password
        - hashes the password
        - creates a user cookie: hash of username+password
        - replaces the / in user cookie: cookies don't recognize slashes
        - inserts the data into the database: sets the pfp to default and money to 1000
    */
    pub fn new_user(&self, username: &str, password: &str) {
        let password_hash = bcrypt::hash(password).unwrap();
        dbg!(&password_hash);
        let cookie = bcrypt::hash(&(username.to_owned() + password))
            .unwrap()
            .replace("/", "a");
        let query = format!("INSERT INTO users (username, password, cookie, profilePicture, money) VALUES ('{username}', '{password_hash}', '{cookie}', '/static/dpfp.png', 1000);");
        self.connection.execute(query).unwrap();
    }

    /* get user
        - data: data known
        - datatype: column to which the data belongs
        - gets user data based on any data assigned to them
        - reads it
        - if exists: convert into a User struct and return
        - if it doesn't exist: return Err
    */
    pub fn get_user(&self, data: &str, datatype: &str) -> sqlite::Result<User> {
        let query = format!("SELECT * FROM users WHERE {datatype}='{data}';");
        let mut statement = self.connection.prepare(query)?;
        statement.next()?;
        let username = statement.read::<String, _>("username")?;
        let password_hash = statement.read::<String, _>("password")?;
        let cookie = statement.read::<String, _>("cookie")?;
        let profile_picture = statement.read::<String, _>("profilePicture")?;
        let money = statement.read::<i64, _>("money")?;

        Ok(User {
            username,
            password_hash,
            cookie,
            profile_picture,
            money,
        })
    }

    /* user update
        - data: new data
        - datatype: what is in need to be updated
        - cookie: to know which user needs updating
        - hashes the data beforehand: lifetime parameter
        - checks if the data is a password: if yes, sets it to its hash
        - updates the database with new data
    */
    pub fn update(&self, data: &str, datatype: &str, cookie: &str, base: &str) {
        let bind = bcrypt::hash(data).unwrap();
        let data = match datatype == "password" {
            true => bind.as_str(),
            false => data,
        };
        let query = format!("UPDATE {base} SET {datatype} = '{data}' WHERE cookie='{cookie}';");
        self.connection.execute(query).unwrap();
    }
    /* new_game
        - host_cookie: cookie of the user creating the game
        - returns doing nothing: a ticket of the same host already exists
        - writes a new game with host_cookie and gust_cookie being null: if there is no such ticket already
    */
    pub fn new_game(&self, host_cookie: &str) -> sqlite::Result<()> {
        let query =
            format!("SELECT EXISTS(SELECT 1 FROM games WHERE host_cookie='{host_cookie}');");
        let mut statement = self.connection.prepare(query)?;
        statement.next()?;
        let exists: bool = statement.read::<i64, _>(0)? != 0;
        if exists {
            return Ok(());
        }
        let query = format!(
            "INSERT INTO games (host_cookie, guest_cookie) VALUES ('{host_cookie}', NULL);"
        );
        self.connection.execute(query)?;
        Ok(())
    }

    /* join open game
        - guest_cookie: joining user cookie
        - checks if a filled game with the user as host exists: returns "/<game row id>/host" is so
        - checks if a game with a different host and empty guest exists: updates it to include the guest_cookie if so
        - gets the game row id of the updated row
        - returns "/<game row id>/guest"
        - if anything of the above is empty returns Err
    */
    pub fn join_open_game(&self, guest_cookie: &str) -> sqlite::Result<String> {
        let query = format!("SELECT rowid, * from games WHERE host_cookie='{guest_cookie}' AND guest_cookie is not NULL");
        let mut statement = self.connection.prepare(query)?;
        statement.next()?;
        let rowid = statement.read::<i64, _>("rowid")?;
        if rowid != 0 {
            return Ok(statement.read::<i64, _>("rowid")?.to_string() + "/host");
        }
        let query = format!(
            "SELECT * FROM games WHERE guest_cookie is NULL AND host_cookie != '{guest_cookie}';"
        );
        let mut statement = self.connection.prepare(query)?;
        statement.next()?;
        let host_cookie = statement.read::<String, _>("host_cookie")?;
        let query = format!(
            "UPDATE games SET guest_cookie = '{guest_cookie}' WHERE host_cookie='{host_cookie}';"
        );
        self.connection.execute(query)?;
        let query = format!("SELECT rowid from games WHERE host_cookie='{host_cookie}' AND guest_cookie = '{guest_cookie}';");
        let mut statement = self.connection.prepare(query)?;
        statement.next()?;
        Ok(statement.read::<i64, _>("rowid")?.to_string() + "/guest")
    }

    /* get opponent cookie
        - typ: what type of player is the player
        - rowid: game row id in the database
        - gets the opposing cookie of the player
        - if player is a host: gets the guest_cookie
        - if player is a guest: gets the host_cookie
        - panics if none of the above
    */
    pub fn get_opponent_cookie(&self, typ: &str, rowid: u64) -> sqlite::Result<String> {
        let query = match typ {
            "host" => format!("SELECT guest_cookie FROM games WHERE rowid='{rowid}';"),
            "guest" => format!("SELECT host_cookie FROM games WHERE rowid='{rowid}';"),
            _ => panic!(),
        };
        let mut statement = self.connection.prepare(query)?;
        statement.next()?;
        Ok(statement.read::<String, _>(0)?)
    }
}

/* password check
    - password_hash: old password hash
    - password: password
    - checks if the password's hash matches the old one: it means its correct
*/
pub fn password_check(password_hash: &String, password: &str) -> bool {
    bcrypt::verify(password, password_hash.as_str())
}
