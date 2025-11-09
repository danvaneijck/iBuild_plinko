use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub token_denom: String,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub treasury_address: Addr,
    pub exchange_rate: Uint128,
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stats {
    pub total_inj_received: Uint128,
    pub total_tokens_minted: Uint128,
    pub total_purchases: u64,
    pub total_house_funding: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATS: Item<Stats> = Item::new("stats");
