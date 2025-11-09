export const NETWORK = import.meta.env.VITE_NETWORK || "testnet";
export const CHAIN_ID = import.meta.env.VITE_CHAIN_ID || "injective-888";
export const RPC_URL =
    import.meta.env.VITE_RPC_URL ||
    "https://testnet.sentry.tm.injective.network:443";
export const REST_URL =
    import.meta.env.VITE_REST_URL ||
    "https://testnet.sentry.lcd.injective.network:443";

export const CONTRACTS = {
    purchase: import.meta.env.VITE_PURCHASE_CONTRACT_ADDRESS || "",
    game: import.meta.env.VITE_GAME_CONTRACT_ADDRESS || "",
    treasury: import.meta.env.VITE_TREASURY_ADDRESS || "",
};

export const TOKEN_SUBDENOM = "plink";
export const TOKEN_DENOM = `factory/${CONTRACTS.purchase}/${TOKEN_SUBDENOM}`;

export const EXCHANGE_RATE = parseInt(
    import.meta.env.VITE_EXCHANGE_RATE || "100"
);

// Validate contract addresses
export const validateContracts = (): boolean => {
    const missing = Object.entries(CONTRACTS)
        .filter(([key, value]) => !value && key !== "treasury")
        .map(([key]) => key);

    if (missing.length > 0) {
        console.warn("Missing contract addresses:", missing);
        return false;
    }

    return true;
};

export const PLINK_TOKEN_DECIMALS = 18;
