import { MsgExecuteContractCompat } from "@injectivelabs/sdk-ts";
import { BigNumberInBase } from "@injectivelabs/utils";
import { WalletStrategy } from "@injectivelabs/wallet-strategy";
import { Network, getNetworkEndpoints } from "@injectivelabs/networks";
import { CONTRACTS, NETWORK, TOKEN_DENOM } from "../config/contracts";
import { Difficulty, LeaderboardType, RiskLevel } from "../types/game";
import { MsgBroadcaster } from "@injectivelabs/wallet-core";

const network = NETWORK === "mainnet" ? Network.Mainnet : Network.Testnet;

const endpoints = getNetworkEndpoints(network);

export class ContractService {
    private walletStrategy: WalletStrategy;
    private msgBroadcaster: MsgBroadcaster;

    constructor(walletStrategy: WalletStrategy) {
        this.walletStrategy = walletStrategy;
        this.msgBroadcaster = new MsgBroadcaster({
            walletStrategy,
            network,
            endpoints,
            simulateTx: true,
            gasBufferCoefficient: 1.2,
        });
    }

    private async createMsgBroadcaster(): Promise<MsgBroadcaster> {
        return new MsgBroadcaster({
            walletStrategy: this.walletStrategy,
            network,
            endpoints,
            // Simulating is crucial for getting gas estimates
            simulateTx: true,
            // A higher gas buffer can prevent out-of-gas errors
            gasBufferCoefficient: 1.4,
        });
    }

    /**
     * Query PLINK token balance for an address
     */
    async getPlinkBalance(address: string): Promise<string> {
        // We now query the native bank module
        const url = `${endpoints.rest}/cosmos/bank/v1beta1/balances/${address}/by_denom?denom=${TOKEN_DENOM}`;

        try {
            const response = await fetch(url);
            if (!response.ok) {
                // If the balance is 0, the endpoint might 404. Treat this as a zero balance.
                if (response.status === 404 || response.status === 500) {
                    return "0.00";
                }
                throw new Error(
                    `Failed to fetch balance: ${response.statusText}`
                );
            }
            const data = await response.json();

            const balance = data?.balance?.amount || "0";

            // Convert from base units (assuming 18 decimals as per your contract)
            return new BigNumberInBase(balance)
                .toWei(-18) // Efficiently divide by 10^18
                .toFixed(2);
        } catch (error) {
            console.error("Error fetching native token balance:", error);
            return "0.00";
        }
    }
    /**
     * Purchase PLINK tokens with INJ
     */
    async purchasePlink(injAmount: string, userAddress: string): Promise<any> {
        try {
            this.msgBroadcaster = new MsgBroadcaster({
                walletStrategy: this.walletStrategy,
                network,
                endpoints,
                simulateTx: true,
                gasBufferCoefficient: 1.2,
            });

            const injectiveAddress = userAddress;

            // Convert INJ amount to base units (18 decimals)
            const amount = new BigNumberInBase(injAmount)
                .times(new BigNumberInBase(10).pow(18))
                .toFixed(0);

            const msg = MsgExecuteContractCompat.fromJSON({
                contractAddress: CONTRACTS.purchase,
                sender: injectiveAddress,
                msg: { purchase: {} },
                funds: {
                    denom: "inj",
                    amount,
                },
            });

            const result = await this.msgBroadcaster.broadcastV2({
                msgs: msg,
                injectiveAddress,
            });

            return result;
        } catch (error) {
            console.error("Error purchasing PLINK:", error);
            throw error;
        }
    }

    /**
     * Query purchase contract configuration
     */
    async getPurchaseConfig(): Promise<any> {
        try {
            const queryMsg = { config: {} };

            const response = await fetch(
                `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
                    CONTRACTS.purchase
                }/smart/${btoa(JSON.stringify(queryMsg))}`
            );

            const data = await response.json();
            return data?.data;
        } catch (error) {
            console.error("Error fetching purchase config:", error);
            return null;
        }
    }

    /**
     * Query purchase statistics
     */
    async getPurchaseStats(): Promise<any> {
        try {
            const queryMsg = { stats: {} };

            const response = await fetch(
                `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
                    CONTRACTS.purchase
                }/smart/${btoa(JSON.stringify(queryMsg))}`
            );

            const data = await response.json();
            return data?.data;
        } catch (error) {
            console.error("Error fetching purchase stats:", error);
            return null;
        }
    }

    /**
     * Play the Plinko game
     */
    async playGame(
        difficulty: Difficulty,
        riskLevel: RiskLevel,
        betAmount: string,
        numberOfPlays: number,
        userAddress: string
    ): Promise<any> {
        const msgBroadcaster = await this.createMsgBroadcaster();

        // Amount for a single play in base units
        const singleBetBaseAmount = new BigNumberInBase(betAmount)
            .toWei(18) // Convert to 18 decimals
            .toFixed(0);

        const messages: MsgExecuteContractCompat[] = [];

        // Create a 'play' message for each ball drop
        for (let i = 0; i < numberOfPlays; i++) {
            const playMsg = MsgExecuteContractCompat.fromJSON({
                contractAddress: CONTRACTS.game,
                sender: userAddress,
                msg: {
                    play: {
                        difficulty: this.mapDifficulty(difficulty),
                        risk_level: this.mapRiskLevel(riskLevel),
                    },
                },
                // Attach the native tokens for this single play
                funds: {
                    denom: TOKEN_DENOM,
                    amount: singleBetBaseAmount,
                },
            });
            messages.push(playMsg);
        }

        // Broadcast the single transaction with all play messages
        const result = await msgBroadcaster.broadcast({
            msgs: messages,
            injectiveAddress: userAddress,
        });

        return result;
    }

    /**
     * [NEW] Query statistics for a specific user
     */
    async getUserStats(userAddress: string): Promise<any> {
        const queryMsg = { user_stats: { player: userAddress } };
        const url = `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
            CONTRACTS.game
        }/smart/${btoa(JSON.stringify(queryMsg))}`;

        try {
            const response = await fetch(url);
            const data = await response.json();
            return data?.data || {}; // Return empty object on failure
        } catch (error) {
            console.error("Error fetching user stats:", error);
            return {};
        }
    }

    /**
     * Query game statistics
     */
    async getGameStats(): Promise<any> {
        try {
            const queryMsg = { stats: {} };

            const response = await fetch(
                `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
                    CONTRACTS.game
                }/smart/${btoa(JSON.stringify(queryMsg))}`
            );

            const data = await response.json();
            return data?.data;
        } catch (error) {
            console.error("Error fetching game stats:", error);
            return null;
        }
    }

    /**
     * Query game history for a player
     */
    async getGameHistory(
        userAddress: string,
        limit: number = 20
    ): Promise<any> {
        try {
            const injectiveAddress = userAddress;
            const queryMsg = {
                history: {
                    player: injectiveAddress,
                    limit,
                },
            };

            const response = await fetch(
                `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
                    CONTRACTS.game
                }/smart/${btoa(JSON.stringify(queryMsg))}`
            );

            const data = await response.json();
            return data?.data?.games || [];
        } catch (error) {
            console.error("Error fetching game history:", error);
            return [];
        }
    }

    /**
     * Query game configuration
     */
    async getGameConfig(): Promise<any> {
        try {
            const queryMsg = { config: {} };

            const response = await fetch(
                `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
                    CONTRACTS.game
                }/smart/${btoa(JSON.stringify(queryMsg))}`
            );

            const data = await response.json();
            return data?.data;
        } catch (error) {
            console.error("Error fetching game config:", error);
            return null;
        }
    }

    // Helper methods to map frontend types to contract types
    private mapDifficulty(difficulty: Difficulty): string {
        const map: Record<Difficulty, string> = {
            easy: "easy",
            medium: "medium",
            hard: "hard",
        };
        return map[difficulty];
    }

    private mapRiskLevel(riskLevel: RiskLevel): string {
        const map: Record<RiskLevel, string> = {
            low: "low",
            medium: "medium",
            high: "high",
        };
        return map[riskLevel];
    }

    /**
     * [NEW] Query the global leaderboard
     */
    async getGlobalLeaderboard(
        leaderboardType: LeaderboardType,
        limit: number = 10
    ): Promise<any[]> {
        const queryMsg = {
            global_leaderboard: {
                leaderboard_type: this.mapLeaderboardType(leaderboardType),
                limit,
            },
        };
        const url = `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
            CONTRACTS.game
        }/smart/${btoa(JSON.stringify(queryMsg))}`;

        try {
            const response = await fetch(url);
            const data = await response.json();
            return data?.data?.entries || [];
        } catch (error) {
            console.error("Error fetching global leaderboard:", error);
            return [];
        }
    }

    /**
     * [NEW] Query the daily leaderboard
     */
    async getDailyLeaderboard(
        leaderboardType: LeaderboardType,
        limit: number = 10
    ): Promise<any[]> {
        const queryMsg = {
            daily_leaderboard: {
                leaderboard_type: this.mapLeaderboardType(leaderboardType),
                limit,
            },
        };
        const url = `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
            CONTRACTS.game
        }/smart/${btoa(JSON.stringify(queryMsg))}`;

        try {
            const response = await fetch(url);
            const data = await response.json();
            return data?.data?.entries || [];
        } catch (error) {
            console.error("Error fetching daily leaderboard:", error);
            return [];
        }
    }

    // Helper to map frontend type to contract's expected enum format
    private mapLeaderboardType(leaderboardType: LeaderboardType): string {
        if (leaderboardType === "bestWins") {
            return "best_wins";
        }
        return "total_wagered";
    }
}
