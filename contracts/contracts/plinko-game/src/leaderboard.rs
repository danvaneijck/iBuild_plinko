use crate::state::LeaderboardEntry;
use cosmwasm_std::{Addr, Uint128};

const MAX_LEADERBOARD_SIZE: usize = 100;

/// Update leaderboard with new entry, maintaining sorted order
pub fn update_leaderboard(
    leaderboard: &mut Vec<LeaderboardEntry>,
    player: Addr,
    value: Uint128,
    multiplier: Option<String>,
) {
    // Remove existing entry for this player if present
    leaderboard.retain(|entry| entry.player != player);

    // Create new entry
    let new_entry = LeaderboardEntry {
        player,
        value,
        multiplier,
    };

    // Find insertion position (descending order)
    let insert_pos = leaderboard
        .iter()
        .position(|entry| entry.value < value)
        .unwrap_or(leaderboard.len());

    // Insert at correct position
    leaderboard.insert(insert_pos, new_entry);

    // Trim to max size
    if leaderboard.len() > MAX_LEADERBOARD_SIZE {
        leaderboard.truncate(MAX_LEADERBOARD_SIZE);
    }
}

/// Check if daily leaderboard needs reset (00:00 UTC)
pub fn should_reset_daily(last_reset_seconds: u64, current_time_seconds: u64) -> bool {
    // The number of seconds in a standard 24-hour day.
    const SECONDS_IN_A_DAY: u64 = 86_400;

    // Calculate the number of full days that have passed since the Unix epoch for each timestamp.
    // Integer division effectively floors the result, giving us a unique "day index".
    let last_reset_day_index = last_reset_seconds / SECONDS_IN_A_DAY;
    let current_day_index = current_time_seconds / SECONDS_IN_A_DAY;

    // If the current day's index is greater than the last reset's day index,
    // it means we have crossed over midnight UTC.
    current_day_index > last_reset_day_index
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Addr;

    #[test]
    fn test_update_leaderboard_new_entry() {
        let mut leaderboard = vec![];

        update_leaderboard(
            &mut leaderboard,
            Addr::unchecked("player1"),
            Uint128::new(100),
            Some("2.0x".to_string()),
        );

        assert_eq!(leaderboard.len(), 1);
        assert_eq!(leaderboard[0].player.as_str(), "player1");
        assert_eq!(leaderboard[0].value, Uint128::new(100));
    }

    #[test]
    fn test_update_leaderboard_sorted_order() {
        let mut leaderboard = vec![];

        update_leaderboard(
            &mut leaderboard,
            Addr::unchecked("player1"),
            Uint128::new(100),
            None,
        );
        update_leaderboard(
            &mut leaderboard,
            Addr::unchecked("player2"),
            Uint128::new(200),
            None,
        );
        update_leaderboard(
            &mut leaderboard,
            Addr::unchecked("player3"),
            Uint128::new(150),
            None,
        );

        assert_eq!(leaderboard.len(), 3);
        assert_eq!(leaderboard[0].value, Uint128::new(200));
        assert_eq!(leaderboard[1].value, Uint128::new(150));
        assert_eq!(leaderboard[2].value, Uint128::new(100));
    }

    #[test]
    fn test_update_leaderboard_replace_existing() {
        let mut leaderboard = vec![];

        update_leaderboard(
            &mut leaderboard,
            Addr::unchecked("player1"),
            Uint128::new(100),
            None,
        );
        update_leaderboard(
            &mut leaderboard,
            Addr::unchecked("player1"),
            Uint128::new(200),
            None,
        );

        assert_eq!(leaderboard.len(), 1);
        assert_eq!(leaderboard[0].value, Uint128::new(200));
    }

    #[test]
    fn test_should_reset_daily_same_day() {
        let base_time = 1704067200; // 2024-01-01 00:00:00 UTC
        let later_same_day = base_time + 3600; // 1 hour later

        assert!(!should_reset_daily(base_time, later_same_day));
    }

    #[test]
    fn test_should_reset_daily_next_day() {
        let base_time = 1704067200; // 2024-01-01 00:00:00 UTC
        let next_day = base_time + 86400; // Next day

        assert!(should_reset_daily(base_time, next_day));
    }

    #[test]
    fn test_should_reset_daily_multiple_days() {
        let base_time = 1704067200; // 2024-01-01 00:00:00 UTC
        let three_days_later = base_time + (86400 * 3);

        assert!(should_reset_daily(base_time, three_days_later));
    }
}
