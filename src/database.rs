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
        - if it doesn't exist: return None
    */
    pub fn get_user(&self, data: &str, datatype: &str) -> Option<User> {
        let query = format!("SELECT * FROM users WHERE {datatype}='{data}';");
        let mut statement = self.connection.prepare(query).unwrap();
        statement.next().unwrap();

        let username = statement.read::<String, _>("username");
        if username.is_err() {
            return None;
        }
        let username = username.unwrap();
        let password_hash = statement.read::<String, _>("password").unwrap();
        let cookie = statement.read::<String, _>("cookie").unwrap();
        let profile_picture = statement.read::<String, _>("profilePicture").unwrap();
        let money = statement.read::<i64, _>("money").unwrap();

        Some(User {
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
    pub fn update_user(&self, data: &str, datatype: &str, cookie: &str) {
        let bind = bcrypt::hash(data).unwrap();
        let data = match datatype == "password" {
            true => bind.as_str(),
            false => data,
        };
        let query = format!("UPDATE users SET {datatype} = '{data}' WHERE cookie='{cookie}';");
        self.connection.execute(query).unwrap();
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
