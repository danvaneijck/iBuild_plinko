use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const MINTER: Item<Addr> = Item::new("minter");
