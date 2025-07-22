pub struct EloRating {
    pub k_factor: f64,
}

impl Default for EloRating {
    fn default() -> Self {
        Self { k_factor: 32.0 }
    }
}

impl EloRating {
    pub fn new(k_factor: f64) -> Self {
        Self { k_factor }
    }

    pub fn expected_score(&self, rating_a: i32, rating_b: i32) -> f64 {
        1.0 / (1.0 + 10.0_f64.powf((rating_b - rating_a) as f64 / 400.0))
    }

    pub fn calculate_new_ratings(
        &self,
        rating_a: i32,
        rating_b: i32,
        score_a: f64,
    ) -> (i32, i32) {
        let expected_a = self.expected_score(rating_a, rating_b);
        let expected_b = 1.0 - expected_a;
        let score_b = 1.0 - score_a;

        let new_rating_a = rating_a + (self.k_factor * (score_a - expected_a)) as i32;
        let new_rating_b = rating_b + (self.k_factor * (score_b - expected_b)) as i32;

        (new_rating_a, new_rating_b)
    }

    pub fn calculate_for_game(&self, winner_elo: i32, loser_elo: i32) -> (i32, i32) {
        self.calculate_new_ratings(winner_elo, loser_elo, 1.0)
    }

    pub fn calculate_for_draw(&self, rating_a: i32, rating_b: i32) -> (i32, i32) {
        self.calculate_new_ratings(rating_a, rating_b, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elo_calculation() {
        let elo = EloRating::default();
        
        let (new_winner, new_loser) = elo.calculate_for_game(1200, 1200);
        assert_eq!(new_winner, 1216);
        assert_eq!(new_loser, 1184);
        
        let (new_winner, new_loser) = elo.calculate_for_game(1400, 1200);
        assert!(new_winner > 1400);
        assert!(new_loser < 1200);
        assert!(new_winner - 1400 < 16);
        assert!(1200 - new_loser < 16);
    }
}