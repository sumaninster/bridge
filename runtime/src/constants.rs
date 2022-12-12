pub mod currency {
    use primitives::types::{Balance, TokenId};

    pub const MILLICENTS: Balance = 10_000_000;
    pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
    pub const DOLLARS: Balance = 100 * CENTS;

    pub const EXISTENTIAL_DEPOSIT: u128 = 10 * CENTS; // 0.1 Native Token Balance

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
    pub const JUR_ASSET_ID: TokenId = 2;
    pub const JUR_MINIMUM_BALANCE: Balance = 0;
    pub const JUR_NAME: &str = "Jur";
    pub const JUR_SYMBOL: &str = "JUR";
    pub const JUR_DECIMALS: u8 = 6;
}