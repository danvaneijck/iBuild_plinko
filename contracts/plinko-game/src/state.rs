use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::GameRecord;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub plink_token_address: Addr,
    pub house_address: Addr,
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stats {
    pub total_games: u64,
    pub total_wagered: Uint128,
    pub total_won: Uint128,
    pub house_balance: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATS: Item<Stats> = Item::new("stats");
pub const GAME_HISTORY: Map<(&Addr, u64), GameRecord> = Map::new("game_history");
pub const PLAYER_GAME_COUNT: Map<&Addr, u64> = Map::new("player_game_count");
