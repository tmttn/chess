//! Chess position evaluation types.

use serde::{Deserialize, Serialize};

/// A chess position evaluation.
///
/// Chess engines report evaluations either as centipawns (1/100th of a pawn)
/// or as mate-in-N moves.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Evaluation {
    /// Centipawn evaluation (positive = white advantage).
    ///
    /// A value of 100 means white is ahead by approximately one pawn.
    Centipawn(i32),
    /// Mate in N moves (positive = white wins, negative = black wins).
    ///
    /// For example, `Mate(3)` means white can force checkmate in 3 moves.
    Mate(i32),
}

impl Evaluation {
    /// Parses evaluation from UCI info score.
    ///
    /// UCI engines report scores as either "score cp `<value>`" or "score mate `<value>`".
    /// If both are provided, mate takes precedence.
    ///
    /// # Arguments
    ///
    /// * `cp` - Centipawn score from UCI (e.g., 35 means +0.35 pawns).
    /// * `mate` - Mate score from UCI (e.g., 3 means mate in 3 moves).
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::Evaluation;
    ///
    /// let eval = Evaluation::from_uci_score(Some(35), None);
    /// assert_eq!(eval, Some(Evaluation::Centipawn(35)));
    ///
    /// let mate = Evaluation::from_uci_score(None, Some(3));
    /// assert_eq!(mate, Some(Evaluation::Mate(3)));
    ///
    /// // Mate takes precedence over cp
    /// let both = Evaluation::from_uci_score(Some(100), Some(3));
    /// assert_eq!(both, Some(Evaluation::Mate(3)));
    /// ```
    pub fn from_uci_score(cp: Option<i32>, mate: Option<i32>) -> Option<Self> {
        if let Some(m) = mate {
            Some(Evaluation::Mate(m))
        } else {
            cp.map(Evaluation::Centipawn)
        }
    }

    /// Returns the centipawn value, converting mate to a large value.
    ///
    /// Mate scores are converted to approximately ±10000 centipawns,
    /// with closer mates having higher absolute values.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::Evaluation;
    ///
    /// assert_eq!(Evaluation::Centipawn(50).to_centipawns(), 50);
    /// assert_eq!(Evaluation::Centipawn(-100).to_centipawns(), -100);
    /// assert!(Evaluation::Mate(1).to_centipawns() > 9000);
    /// assert!(Evaluation::Mate(-1).to_centipawns() < -9000);
    /// ```
    pub fn to_centipawns(&self) -> i32 {
        match self {
            Evaluation::Centipawn(cp) => *cp,
            Evaluation::Mate(n) => {
                if *n > 0 {
                    10000 - (*n * 10) // Closer mate = higher score
                } else {
                    -10000 - (*n * 10) // Closer mate = lower score
                }
            }
        }
    }

    /// Returns true if this evaluation is better for white than the other.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::Evaluation;
    ///
    /// let good = Evaluation::Centipawn(100);
    /// let bad = Evaluation::Centipawn(-50);
    /// assert!(good.is_better_for_white(&bad));
    /// assert!(!bad.is_better_for_white(&good));
    /// ```
    pub fn is_better_for_white(&self, other: &Evaluation) -> bool {
        self.to_centipawns() > other.to_centipawns()
    }

    /// Returns true if this evaluation is better for black than the other.
    pub fn is_better_for_black(&self, other: &Evaluation) -> bool {
        self.to_centipawns() < other.to_centipawns()
    }

    /// Returns the evaluation from black's perspective (negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::Evaluation;
    ///
    /// assert_eq!(Evaluation::Centipawn(50).flip(), Evaluation::Centipawn(-50));
    /// assert_eq!(Evaluation::Mate(3).flip(), Evaluation::Mate(-3));
    /// ```
    pub fn flip(&self) -> Self {
        match self {
            Evaluation::Centipawn(cp) => Evaluation::Centipawn(-cp),
            Evaluation::Mate(n) => Evaluation::Mate(-n),
        }
    }
}

impl std::fmt::Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::Centipawn(cp) => {
                let sign = if *cp >= 0 { "+" } else { "" };
                write!(f, "{}{:.2}", sign, *cp as f32 / 100.0)
            }
            Evaluation::Mate(n) => {
                if *n > 0 {
                    write!(f, "#{}", n)
                } else {
                    write!(f, "#-{}", -n)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_uci_score_centipawn() {
        let eval = Evaluation::from_uci_score(Some(35), None);
        assert_eq!(eval, Some(Evaluation::Centipawn(35)));
    }

    #[test]
    fn test_from_uci_score_negative_centipawn() {
        let eval = Evaluation::from_uci_score(Some(-150), None);
        assert_eq!(eval, Some(Evaluation::Centipawn(-150)));
    }

    #[test]
    fn test_from_uci_score_mate() {
        let eval = Evaluation::from_uci_score(None, Some(3));
        assert_eq!(eval, Some(Evaluation::Mate(3)));
    }

    #[test]
    fn test_from_uci_score_negative_mate() {
        let eval = Evaluation::from_uci_score(None, Some(-5));
        assert_eq!(eval, Some(Evaluation::Mate(-5)));
    }

    #[test]
    fn test_from_uci_score_mate_takes_precedence() {
        let eval = Evaluation::from_uci_score(Some(100), Some(3));
        assert_eq!(eval, Some(Evaluation::Mate(3))); // Mate takes precedence
    }

    #[test]
    fn test_from_uci_score_none() {
        let eval = Evaluation::from_uci_score(None, None);
        assert_eq!(eval, None);
    }

    #[test]
    fn test_to_centipawns_basic() {
        assert_eq!(Evaluation::Centipawn(50).to_centipawns(), 50);
        assert_eq!(Evaluation::Centipawn(-100).to_centipawns(), -100);
        assert_eq!(Evaluation::Centipawn(0).to_centipawns(), 0);
    }

    #[test]
    fn test_to_centipawns_mate() {
        // Mate in 1 should be very high
        assert!(Evaluation::Mate(1).to_centipawns() > 9000);
        // Mate in 10 should be lower than mate in 1
        assert!(Evaluation::Mate(1).to_centipawns() > Evaluation::Mate(10).to_centipawns());
        // Negative mate (opponent wins) should be very low
        assert!(Evaluation::Mate(-1).to_centipawns() < -9000);
    }

    #[test]
    fn test_display_centipawn() {
        assert_eq!(format!("{}", Evaluation::Centipawn(35)), "+0.35");
        assert_eq!(format!("{}", Evaluation::Centipawn(-150)), "-1.50");
        assert_eq!(format!("{}", Evaluation::Centipawn(0)), "+0.00");
        assert_eq!(format!("{}", Evaluation::Centipawn(100)), "+1.00");
    }

    #[test]
    fn test_display_mate() {
        assert_eq!(format!("{}", Evaluation::Mate(3)), "#3");
        assert_eq!(format!("{}", Evaluation::Mate(1)), "#1");
        assert_eq!(format!("{}", Evaluation::Mate(-2)), "#-2");
        assert_eq!(format!("{}", Evaluation::Mate(-10)), "#-10");
    }

    #[test]
    fn test_is_better_for_white() {
        let good = Evaluation::Centipawn(100);
        let bad = Evaluation::Centipawn(-50);
        let mate = Evaluation::Mate(3);

        assert!(good.is_better_for_white(&bad));
        assert!(!bad.is_better_for_white(&good));
        assert!(mate.is_better_for_white(&good));
    }

    #[test]
    fn test_is_better_for_black() {
        let good_for_white = Evaluation::Centipawn(100);
        let good_for_black = Evaluation::Centipawn(-200);

        assert!(good_for_black.is_better_for_black(&good_for_white));
        assert!(!good_for_white.is_better_for_black(&good_for_black));
    }

    #[test]
    fn test_flip() {
        assert_eq!(Evaluation::Centipawn(50).flip(), Evaluation::Centipawn(-50));
        assert_eq!(
            Evaluation::Centipawn(-100).flip(),
            Evaluation::Centipawn(100)
        );
        assert_eq!(Evaluation::Mate(3).flip(), Evaluation::Mate(-3));
        assert_eq!(Evaluation::Mate(-5).flip(), Evaluation::Mate(5));
    }

    #[test]
    fn test_clone_and_copy() {
        let eval = Evaluation::Centipawn(50);
        let cloned = eval.clone();
        let copied = eval;
        assert_eq!(eval, cloned);
        assert_eq!(eval, copied);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let eval = Evaluation::Centipawn(123);
        let json = serde_json::to_string(&eval).unwrap();
        let parsed: Evaluation = serde_json::from_str(&json).unwrap();
        assert_eq!(eval, parsed);

        let mate = Evaluation::Mate(-5);
        let json = serde_json::to_string(&mate).unwrap();
        let parsed: Evaluation = serde_json::from_str(&json).unwrap();
        assert_eq!(mate, parsed);
    }
}
