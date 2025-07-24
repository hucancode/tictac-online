/// Type-safe SurrealQL query for user profile with game statistics
pub const USER_PROFILE_QUERY: &str = r#"
    -- Define the user record type
    LET $uid = type::record('user', $user_id);
    
    -- Get user data
    LET $user = (SELECT * FROM $uid)[0];
    
    -- Count games efficiently
    LET $games = SELECT 
        count() as total,
        count(winner = $uid) as won
    FROM game 
    WHERE (player1 = $uid OR player2 = $uid) 
    AND status = 'completed';
    
    -- Return strongly typed profile
    RETURN {
        id: string::concat('user:', $user_id),
        email: $user.email,
        username: $user.username,
        profile_picture: $user.profile_picture,
        elo: $user.elo,
        games_played: $games[0].total OR 0,
        games_won: $games[0].won OR 0,
        win_rate: IF $games[0].total > 0 THEN 
            math::round($games[0].won * 100.0 / $games[0].total, 2) 
        ELSE 0.0 END
    };
"#;