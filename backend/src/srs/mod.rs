//! Anki-variant SM-2 update rule. Pure functions only — no I/O. The route
//! layer reads `user_case_progress`, calls `next_state`, writes the result.
//!
//! Authoritative spec: docs/Cube_Practice_Design_Doc.md §4 and
//! docs/milestones/03_core_study_loop.md §4. The grading scale is 0..=3
//! (Fail / Hard / Good / Easy); see `Grade`.

use chrono::{Days, NaiveDate};

pub const INITIAL_EASE: f64 = 2.5;
pub const EASE_FLOOR: f64 = 1.3;
pub const HARD_INTERVAL_MULT: f64 = 1.2;
pub const EASY_BONUS: f64 = 1.3;
pub const FAIL_EASE_DELTA: f64 = -0.20;
pub const HARD_EASE_DELTA: f64 = -0.15;
pub const EASY_EASE_DELTA: f64 = 0.15;

/// SM-2 row state. The pure `next_state` consumes one and returns the next.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressState {
    pub ease_factor: f64,
    pub interval_days: i32,
    pub repetitions: i32,
    pub due_date: NaiveDate,
}

impl ProgressState {
    /// Initial state for a brand-new card. `due_date = today` so the first
    /// review (which calls `next_state` on this) treats today as the
    /// review day.
    pub fn initial(today: NaiveDate) -> Self {
        Self {
            ease_factor: INITIAL_EASE,
            interval_days: 1,
            repetitions: 0,
            due_date: today,
        }
    }
}

/// 4-button grade: Fail (0), Hard (1), Good (2), Easy (3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Fail,
    Hard,
    Good,
    Easy,
}

impl Grade {
    pub fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(Grade::Fail),
            1 => Some(Grade::Hard),
            2 => Some(Grade::Good),
            3 => Some(Grade::Easy),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Grade::Fail => 0,
            Grade::Hard => 1,
            Grade::Good => 2,
            Grade::Easy => 3,
        }
    }
}

/// Apply the grade to `prev` using the Anki-variant SM-2 rule. Pure;
/// caller persists the returned state.
pub fn next_state(prev: ProgressState, grade: Grade, today: NaiveDate) -> ProgressState {
    let ProgressState {
        mut ease_factor,
        mut interval_days,
        mut repetitions,
        ..
    } = prev;

    if grade == Grade::Fail {
        repetitions = 0;
        interval_days = 1;
        ease_factor += FAIL_EASE_DELTA;
    } else {
        interval_days = match repetitions {
            0 => 1,
            1 => 6,
            _ => match grade {
                Grade::Hard => round_i32(interval_days as f64 * HARD_INTERVAL_MULT),
                Grade::Good => round_i32(interval_days as f64 * ease_factor),
                Grade::Easy => round_i32(interval_days as f64 * ease_factor * EASY_BONUS),
                Grade::Fail => unreachable!("handled in outer if"),
            },
        };
        repetitions += 1;
        match grade {
            Grade::Hard => ease_factor += HARD_EASE_DELTA,
            Grade::Good => {}
            Grade::Easy => ease_factor += EASY_EASE_DELTA,
            Grade::Fail => unreachable!(),
        }
    }

    if ease_factor < EASE_FLOOR {
        ease_factor = EASE_FLOOR;
    }
    if interval_days < 1 {
        interval_days = 1;
    }

    let due_date = today
        .checked_add_days(Days::new(interval_days as u64))
        .expect("interval too large to add — should never happen for SM-2 in practice");

    ProgressState {
        ease_factor,
        interval_days,
        repetitions,
        due_date,
    }
}

fn round_i32(v: f64) -> i32 {
    v.round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn at_initial(today: NaiveDate) -> ProgressState {
        ProgressState::initial(today)
    }

    // ─── Fail ────────────────────────────────────────────────────────────────

    #[test]
    fn fail_at_rep_0_keeps_rep_0() {
        let today = day("2026-04-29");
        let next = next_state(at_initial(today), Grade::Fail, today);
        assert_eq!(next.repetitions, 0);
        assert_eq!(next.interval_days, 1);
        assert!((next.ease_factor - 2.30).abs() < 1e-9);
        assert_eq!(next.due_date, day("2026-04-30"));
    }

    #[test]
    fn fail_after_streak_resets_reps_and_interval() {
        let prev = ProgressState {
            ease_factor: 2.4,
            interval_days: 30,
            repetitions: 5,
            due_date: day("2026-05-29"),
        };
        let today = day("2026-04-29");
        let next = next_state(prev, Grade::Fail, today);
        assert_eq!(next.repetitions, 0);
        assert_eq!(next.interval_days, 1);
        assert!((next.ease_factor - 2.20).abs() < 1e-9);
        assert_eq!(next.due_date, day("2026-04-30"));
    }

    #[test]
    fn fail_does_not_drop_ease_below_floor() {
        let prev = ProgressState {
            ease_factor: EASE_FLOOR,
            interval_days: 3,
            repetitions: 2,
            due_date: day("2026-05-02"),
        };
        let today = day("2026-04-29");
        let next = next_state(prev, Grade::Fail, today);
        assert!((next.ease_factor - EASE_FLOOR).abs() < 1e-9);
    }

    // ─── Good ────────────────────────────────────────────────────────────────

    #[test]
    fn good_at_rep_0_advances_to_rep_1_interval_1() {
        let today = day("2026-04-29");
        let next = next_state(at_initial(today), Grade::Good, today);
        assert_eq!(next.repetitions, 1);
        assert_eq!(next.interval_days, 1);
        assert!((next.ease_factor - INITIAL_EASE).abs() < 1e-9);
        assert_eq!(next.due_date, day("2026-04-30"));
    }

    #[test]
    fn good_at_rep_1_advances_to_rep_2_interval_6() {
        let prev = ProgressState {
            ease_factor: 2.5,
            interval_days: 1,
            repetitions: 1,
            due_date: day("2026-04-30"),
        };
        let today = day("2026-04-30");
        let next = next_state(prev, Grade::Good, today);
        assert_eq!(next.repetitions, 2);
        assert_eq!(next.interval_days, 6);
        assert!((next.ease_factor - 2.5).abs() < 1e-9);
        assert_eq!(next.due_date, day("2026-05-06"));
    }

    #[test]
    fn good_at_rep_2_multiplies_by_ease() {
        let prev = ProgressState {
            ease_factor: 2.5,
            interval_days: 6,
            repetitions: 2,
            due_date: day("2026-05-06"),
        };
        let today = day("2026-05-06");
        let next = next_state(prev, Grade::Good, today);
        assert_eq!(next.repetitions, 3);
        assert_eq!(next.interval_days, 15); // round(6 * 2.5)
        assert!((next.ease_factor - 2.5).abs() < 1e-9);
        assert_eq!(next.due_date, day("2026-05-21"));
    }

    // ─── Hard ────────────────────────────────────────────────────────────────

    #[test]
    fn hard_at_rep_0_acts_like_first_review_with_ease_drop() {
        let today = day("2026-04-29");
        let next = next_state(at_initial(today), Grade::Hard, today);
        assert_eq!(next.repetitions, 1);
        assert_eq!(next.interval_days, 1);
        assert!((next.ease_factor - 2.35).abs() < 1e-9);
    }

    #[test]
    fn hard_at_rep_2_uses_hard_multiplier_and_ease_drops() {
        let prev = ProgressState {
            ease_factor: 2.5,
            interval_days: 10,
            repetitions: 2,
            due_date: day("2026-05-09"),
        };
        let today = day("2026-05-09");
        let next = next_state(prev, Grade::Hard, today);
        assert_eq!(next.repetitions, 3);
        assert_eq!(next.interval_days, 12); // round(10 * 1.2)
        assert!((next.ease_factor - 2.35).abs() < 1e-9);
    }

    // ─── Easy ────────────────────────────────────────────────────────────────

    #[test]
    fn easy_at_rep_2_applies_easy_bonus_and_ease_rises() {
        let prev = ProgressState {
            ease_factor: 2.5,
            interval_days: 10,
            repetitions: 2,
            due_date: day("2026-05-09"),
        };
        let today = day("2026-05-09");
        let next = next_state(prev, Grade::Easy, today);
        assert_eq!(next.repetitions, 3);
        // round(10 * 2.5 * 1.3) = round(32.5) = 33 (banker's rounding lands on 33 for 32.5? no — Rust f64::round() rounds half away from zero, so 32.5 → 33)
        assert_eq!(next.interval_days, 33);
        assert!((next.ease_factor - 2.65).abs() < 1e-9);
    }

    // ─── Floor + interval guard ──────────────────────────────────────────────

    #[test]
    fn ease_floor_holds_through_consecutive_hards() {
        let mut state = ProgressState {
            ease_factor: 1.4,
            interval_days: 4,
            repetitions: 3,
            due_date: day("2026-05-03"),
        };
        let today = day("2026-05-03");
        // Hard drops ease by 0.15 — would land at 1.25, clamped to 1.3.
        state = next_state(state, Grade::Hard, today);
        assert!((state.ease_factor - EASE_FLOOR).abs() < 1e-9);
        // Another Hard — ease stays at floor.
        state = next_state(state, Grade::Hard, today);
        assert!((state.ease_factor - EASE_FLOOR).abs() < 1e-9);
    }

    // ─── Round trip ──────────────────────────────────────────────────────────

    #[test]
    fn round_trip_good_good_good_matches_expected_sequence() {
        // Three Good reviews from initial: rep 0→1 (interval 1), 1→2 (6), 2→3 (15).
        let mut today = day("2026-04-29");
        let mut state = ProgressState::initial(today);

        state = next_state(state, Grade::Good, today);
        assert_eq!((state.repetitions, state.interval_days), (1, 1));
        today = state.due_date;

        state = next_state(state, Grade::Good, today);
        assert_eq!((state.repetitions, state.interval_days), (2, 6));
        today = state.due_date;

        state = next_state(state, Grade::Good, today);
        assert_eq!((state.repetitions, state.interval_days), (3, 15));
    }

    // ─── Grade enum ──────────────────────────────────────────────────────────

    #[test]
    fn grade_from_u8_round_trips() {
        for n in 0u8..=3 {
            let g = Grade::from_u8(n).unwrap();
            assert_eq!(g.as_u8(), n);
        }
    }

    #[test]
    fn grade_from_u8_rejects_out_of_range() {
        assert!(Grade::from_u8(4).is_none());
        assert!(Grade::from_u8(255).is_none());
    }
}
