use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub plink_token_address: Addr,
    pub treasury_address: Addr,
    pub exchange_rate: Uint128,
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stats {
    pub total_inj_received: Uint128,
    pub total_plink_minted: Uint128,
    pub total_purchases: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATS: Item<Stats> = Item::new("stats");
