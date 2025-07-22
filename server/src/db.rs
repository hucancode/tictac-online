use once_cell::sync::OnceCell;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use std::env;
use crate::models::User;
use bcrypt::{hash, DEFAULT_COST};

static DB: OnceCell<Surreal<Client>> = OnceCell::new();

pub async fn init_db() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "ws://localhost:8000".to_string());
    let db_host = db_url.replace("ws://", "");
    
    println!("Connecting to SurrealDB at {}", db_host);
    let db = Surreal::new::<Ws>(db_host.as_str()).await?;
    println!("Connected to SurrealDB");
    
    println!("Signing in...");
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;
    println!("Signed in successfully");
    
    println!("Selecting namespace and database...");
    db.use_ns("tictac").use_db("tictac").await?;
    println!("Selected namespace and database");
    
    println!("Creating tables...");
    create_tables(&db).await?;
    println!("Tables created");
    
    DB.set(db).map_err(|_| "Failed to set database connection")?;
    
    // Create default admin user if it doesn't exist
    create_default_admin().await?;
    
    Ok(())
}

async fn create_tables(db: &Surreal<Client>) -> Result<(), Box<dyn std::error::Error>> {
    db.query(r#"
        DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS email ON TABLE user TYPE string ASSERT string::is::email($value);
        DEFINE FIELD IF NOT EXISTS username ON TABLE user TYPE string;
        DEFINE FIELD IF NOT EXISTS password_hash ON TABLE user TYPE string;
        DEFINE FIELD IF NOT EXISTS profile_picture ON TABLE user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS elo ON TABLE user TYPE int DEFAULT 1200;
        DEFINE FIELD IF NOT EXISTS games_played ON TABLE user TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS games_won ON TABLE user TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS is_admin ON TABLE user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS created_at ON TABLE user TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS updated_at ON TABLE user TYPE datetime DEFAULT time::now();
        DEFINE INDEX IF NOT EXISTS email_idx ON TABLE user COLUMNS email UNIQUE;
        DEFINE INDEX IF NOT EXISTS username_idx ON TABLE user COLUMNS username UNIQUE;
        
        DEFINE TABLE IF NOT EXISTS game SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS player1 ON TABLE game TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS player2 ON TABLE game TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS winner ON TABLE game TYPE option<record<user>>;
        DEFINE FIELD IF NOT EXISTS board ON TABLE game TYPE array;
        DEFINE FIELD IF NOT EXISTS status ON TABLE game TYPE string DEFAULT 'waiting';
        DEFINE FIELD IF NOT EXISTS player1_elo_before ON TABLE game TYPE int;
        DEFINE FIELD IF NOT EXISTS player2_elo_before ON TABLE game TYPE int;
        DEFINE FIELD IF NOT EXISTS player1_elo_after ON TABLE game TYPE option<int>;
        DEFINE FIELD IF NOT EXISTS player2_elo_after ON TABLE game TYPE option<int>;
        DEFINE FIELD IF NOT EXISTS started_at ON TABLE game TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS ended_at ON TABLE game TYPE option<datetime>;
        
        DEFINE TABLE IF NOT EXISTS session SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS user ON TABLE session TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS token ON TABLE session TYPE string;
        DEFINE FIELD IF NOT EXISTS expires_at ON TABLE session TYPE datetime;
        DEFINE FIELD IF NOT EXISTS created_at ON TABLE session TYPE datetime DEFAULT time::now();
        DEFINE INDEX IF NOT EXISTS token_idx ON TABLE session COLUMNS token UNIQUE;
    "#)
    .await?
    .check()?;
    
    Ok(())
}

pub fn get_db() -> &'static Surreal<Client> {
    DB.get().expect("Database not initialized")
}

async fn create_default_admin() -> Result<(), Box<dyn std::error::Error>> {
    let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
    let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    
    // Check if admin user already exists
    let mut result = get_db()
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", admin_email.clone()))
        .await?;
    
    let existing_users: Vec<User> = result.take(0)?;
    
    if existing_users.is_empty() {
        println!("Creating default admin user...");
        
        let password_hash = hash(admin_password.as_bytes(), DEFAULT_COST)?;
        
        let _: Option<User> = get_db()
            .create("user")
            .content(serde_json::json!({
                "email": admin_email.clone(),
                "username": "admin",
                "password_hash": password_hash,
                "profile_picture": null,
                "elo": 1200,
                "games_played": 0,
                "games_won": 0,
                "is_admin": true
            }))
            .await?;
        
        println!("Default admin user created with email: {}", admin_email);
    } else {
        println!("Admin user already exists, skipping creation");
    }
    
    Ok(())
}