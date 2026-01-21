//! Elo rating calculation.
//!
//! This module provides functions for calculating Elo ratings after games.
//! The K-factor of 32 is used, which is standard for most chess rating systems.

const K_FACTOR: f64 = 32.0;

/// Calculate expected score for player A against player B.
fn expected_score(rating_a: i32, rating_b: i32) -> f64 {
    1.0 / (1.0 + 10_f64.powf((rating_b - rating_a) as f64 / 400.0))
}

/// Calculate new rating after a game.
///
/// # Arguments
/// * `rating` - Current rating
/// * `opponent_rating` - Opponent's rating
/// * `actual` - Actual score (1.0 = win, 0.5 = draw, 0.0 = loss)
pub fn new_rating(rating: i32, opponent_rating: i32, actual: f64) -> i32 {
    let expected = expected_score(rating, opponent_rating);
    let new = rating as f64 + K_FACTOR * (actual - expected);
    new.round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_score_equal_ratings() {
        let expected = expected_score(1500, 1500);
        assert!((expected - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_expected_score_higher_rated() {
        let expected = expected_score(1700, 1500);
        assert!(expected > 0.7);
        assert!(expected < 0.8);
    }

    #[test]
    fn test_expected_score_lower_rated() {
        let expected = expected_score(1300, 1500);
        assert!(expected < 0.3);
        assert!(expected > 0.2);
    }

    #[test]
    fn test_new_rating_win() {
        let new = new_rating(1500, 1500, 1.0);
        assert_eq!(new, 1516);
    }

    #[test]
    fn test_new_rating_loss() {
        let new = new_rating(1500, 1500, 0.0);
        assert_eq!(new, 1484);
    }

    #[test]
    fn test_new_rating_draw() {
        let new = new_rating(1500, 1500, 0.5);
        assert_eq!(new, 1500);
    }

    #[test]
    fn test_new_rating_upset_win() {
        // Lower rated player wins
        let new = new_rating(1300, 1500, 1.0);
        assert!(new > 1320); // Bigger gain for upset
    }
}
