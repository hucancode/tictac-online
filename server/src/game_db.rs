use crate::{
    db::get_db,
    models::{GameRecord, User},
};
use surrealdb::RecordId;

pub async fn create_game(player1_email: &str, player2_email: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Creating game between {} and {}", player1_email, player2_email);
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

    // Debug: Print what we're sending
    println!("Player1 ID: {:?}", player1.id);
    println!("Player2 ID: {:?}", player2.id);

    // Extract the ID strings for use in type::thing
    let player1_id = player1.id.as_ref().unwrap().to_string();
    let player2_id = player2.id.as_ref().unwrap().to_string();

    // Extract just the ID part after "user:"
    let player1_id_clean = player1_id.strip_prefix("user:").unwrap_or(&player1_id).to_string();
    let player2_id_clean = player2_id.strip_prefix("user:").unwrap_or(&player2_id).to_string();

    // Generate a game ID that we control
    let game_id = format!("{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());

    // Use a query to create game with specific ID
    let result = get_db()
        .query(r#"
            CREATE type::thing('game', $game_id) CONTENT {
                player1: type::thing('user', $player1_id),
                player2: type::thing('user', $player2_id),
                winner: NONE,
                board: $board,
                status: "active",
                player1_elo_before: $elo1,
                player2_elo_before: $elo2,
                player1_elo_after: NONE,
                player2_elo_after: NONE,
                started_at: time::now(),
                ended_at: NONE
            };
        "#)
        .bind(("game_id", game_id.clone()))
        .bind(("player1_id", player1_id_clean))
        .bind(("player2_id", player2_id_clean))
        .bind(("board", vec![vec![None::<i32>; 10]; 10]))
        .bind(("elo1", player1.elo))
        .bind(("elo2", player2.elo))
        .await;

    // Debug: Check what error we're getting
    match result {
        Ok(response) => {
            println!("Query succeeded, trying to parse response...");
            // Don't try to deserialize, just log it
            println!("Response: {:?}", response);

            println!("Game created successfully with ID: {}", game_id);
            Ok(game_id)
        }
        Err(e) => {
            println!("Query failed with error: {:?}", e);
            Err(Box::new(e))
        }
    }
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
    println!("Ending game {} with winner: {:?}", game_id, winner_email);
    
    let query = if let Some(_winner_email) = winner_email {
        // Query for when there's a winner
        r#"
        BEGIN TRANSACTION;
        
        -- Get the game
        LET $game = (SELECT * FROM type::thing('game', $game_id))[0];
        
        -- Get the players
        LET $p1 = (SELECT * FROM $game.player1)[0];
        LET $p2 = (SELECT * FROM $game.player2)[0];
        
        -- Determine winner
        LET $winner_id = IF ($p1.email = $winner_email) THEN $p1.id ELSE $p2.id END;
        LET $is_p1_winner = ($p1.email = $winner_email);
        
        -- Calculate new ELOs (K=32, winner gets +16, loser gets -16)
        LET $new_elo1 = IF ($is_p1_winner) THEN ($p1.elo + 16) ELSE ($p1.elo - 16) END;
        LET $new_elo2 = IF ($is_p1_winner) THEN ($p2.elo - 16) ELSE ($p2.elo + 16) END;
        
        -- Update the game
        UPDATE type::thing('game', $game_id) SET
            status = 'completed',
            winner = $winner_id,
            player1_elo_after = $new_elo1,
            player2_elo_after = $new_elo2,
            ended_at = time::now();
        
        -- Update player 1
        UPDATE $p1.id SET
            elo = $new_elo1,
            updated_at = time::now();
            
        -- Update player 2
        UPDATE $p2.id SET
            elo = $new_elo2,
            updated_at = time::now();
            
        COMMIT TRANSACTION;
            
        -- Return confirmation
        RETURN {
            game_id: $game_id,
            winner: $winner_id,
            p1_elo: { before: $p1.elo, after: $new_elo1 },
            p2_elo: { before: $p2.elo, after: $new_elo2 }
        };
        "#
    } else {
        // Query for draw (no winner)
        r#"
        BEGIN TRANSACTION;
        
        -- Get the game
        LET $game = (SELECT * FROM type::thing('game', $game_id))[0];
        
        -- Get the players
        LET $p1 = (SELECT * FROM $game.player1)[0];
        LET $p2 = (SELECT * FROM $game.player2)[0];
        
        -- Update the game (draw - no ELO change)
        UPDATE type::thing('game', $game_id) SET
            status = 'completed',
            winner = NONE,
            player1_elo_after = $p1.elo,
            player2_elo_after = $p2.elo,
            ended_at = time::now();
        
        -- Update player timestamps
        UPDATE $p1.id SET updated_at = time::now();
        UPDATE $p2.id SET updated_at = time::now();
        
        COMMIT TRANSACTION;
            
        -- Return confirmation
        RETURN {
            game_id: $game_id,
            winner: NONE,
            p1_elo: { before: $p1.elo, after: $p1.elo },
            p2_elo: { before: $p2.elo, after: $p2.elo }
        };
        "#
    };
    
    // Execute the query
    let _result = if let Some(winner_email) = winner_email {
        get_db()
            .query(query)
            .bind(("game_id", game_id.to_string()))
            .bind(("winner_email", winner_email.to_string()))
            .await?
    } else {
        get_db()
            .query(query)
            .bind(("game_id", game_id.to_string()))
            .await?
    };
    
    // The transaction succeeded if we got here without error
    // We don't need to process individual results since the transaction
    // either completes fully or rolls back
    println!("Game {} ended successfully!", game_id);
    Ok(())
}