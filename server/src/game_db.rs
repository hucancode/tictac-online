use crate::{
    db::get_db,
    elo::EloRating,
    models::{GameRecord, User},
};
use chrono::Utc;
use surrealdb::RecordId;

pub async fn create_game(player1_email: &str, player2_email: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Get both players
    let mut result = get_db()
        .query("SELECT * FROM user WHERE email = $email1 OR email = $email2")
        .bind(("email1", player1_email.to_string()))
        .bind(("email2", player2_email.to_string()))
        .await?;
    
    let users: Vec<User> = result.take(0)?;
    
    if users.len() != 2 {
        return Err("Could not find both players".into());
    }
    
    let (player1, player2) = if users[0].email == player1_email {
        (&users[0], &users[1])
    } else {
        (&users[1], &users[0])
    };
    
    let game = GameRecord {
        id: None,
        player1: player1.id.as_ref().unwrap().clone(),
        player2: player2.id.as_ref().unwrap().clone(),
        winner: None,
        board: vec![vec![None; 10]; 10],
        status: "active".to_string(),
        player1_elo_before: player1.elo,
        player2_elo_before: player2.elo,
        player1_elo_after: None,
        player2_elo_after: None,
        started_at: Utc::now(),
        ended_at: None,
    };
    
    let created: Option<GameRecord> = get_db()
        .create("game")
        .content(game.clone())
        .await?;
    
    let game = created.ok_or("Failed to create game")?;
    
    Ok(game.id.as_ref().unwrap().to_string())
}

pub async fn update_game_board(game_id: &str, board: Vec<Vec<Option<i32>>>) -> Result<(), Box<dyn std::error::Error>> {
    let game_thing = RecordId::from(("game", game_id));
    
    let _: Option<GameRecord> = get_db()
        .update(game_thing)
        .merge(serde_json::json!({
            "board": board,
        }))
        .await?;
    
    Ok(())
}

pub async fn end_game(
    game_id: &str,
    winner_email: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let game_thing = RecordId::from(("game", game_id));
    
    // Get the game record
    let game: Option<GameRecord> = get_db().select(game_thing.clone()).await?;
    let game = game.ok_or("Game not found")?;
    
    // Get both players
    let player1: Option<User> = get_db().select(game.player1.clone()).await?;
    let player2: Option<User> = get_db().select(game.player2.clone()).await?;
    
    let player1 = player1.ok_or("Player 1 not found")?;
    let player2 = player2.ok_or("Player 2 not found")?;
    
    let elo_calc = EloRating::default();
    
    let (new_elo1, new_elo2, winner_thing) = if let Some(winner_email) = winner_email {
        if player1.email == winner_email {
            let (winner_elo, loser_elo) = elo_calc.calculate_for_game(player1.elo, player2.elo);
            (winner_elo, loser_elo, Some(player1.id.as_ref().unwrap().clone()))
        } else {
            let (winner_elo, loser_elo) = elo_calc.calculate_for_game(player2.elo, player1.elo);
            (loser_elo, winner_elo, Some(player2.id.as_ref().unwrap().clone()))
        }
    } else {
        // Draw
        let (elo1, elo2) = elo_calc.calculate_for_draw(player1.elo, player2.elo);
        (elo1, elo2, None)
    };
    
    // Update game record
    let _: Option<GameRecord> = get_db()
        .update(game_thing)
        .merge(serde_json::json!({
            "status": "completed",
            "winner": winner_thing,
            "player1_elo_after": new_elo1,
            "player2_elo_after": new_elo2,
            "ended_at": Utc::now(),
        }))
        .await?;
    
    // Update player 1 stats
    let games_won1 = if winner_email == Some(&player1.email) {
        player1.games_won + 1
    } else {
        player1.games_won
    };
    
    let _: Option<User> = get_db()
        .update(player1.id.as_ref().unwrap().clone())
        .merge(serde_json::json!({
            "elo": new_elo1,
            "games_played": player1.games_played + 1,
            "games_won": games_won1,
            "updated_at": Utc::now(),
        }))
        .await?;
    
    // Update player 2 stats
    let games_won2 = if winner_email == Some(&player2.email) {
        player2.games_won + 1
    } else {
        player2.games_won
    };
    
    let _: Option<User> = get_db()
        .update(player2.id.as_ref().unwrap().clone())
        .merge(serde_json::json!({
            "elo": new_elo2,
            "games_played": player2.games_played + 1,
            "games_won": games_won2,
            "updated_at": Utc::now(),
        }))
        .await?;
    
    Ok(())
}