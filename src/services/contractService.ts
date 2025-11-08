import { MsgExecuteContractCompat } from "@injectivelabs/sdk-ts";
import { BigNumberInBase } from "@injectivelabs/utils";
import { WalletStrategy } from "@injectivelabs/wallet-strategy";
import { Network, getNetworkEndpoints } from "@injectivelabs/networks";
import { ChainId } from "@injectivelabs/ts-types";
import { CONTRACTS, NETWORK, CHAIN_ID } from "../config/contracts";
import { Difficulty, RiskLevel } from "../types/game";
import { MsgBroadcaster } from "@injectivelabs/wallet-core";

const network = NETWORK === "mainnet" ? Network.Mainnet : Network.Testnet;
const chainId = CHAIN_ID === "injective-1" ? ChainId.Mainnet : ChainId.Testnet;

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

    /**
     * Query PLINK token balance for an address
     */
    async getPlinkBalance(address: string): Promise<string> {
        try {
            const injectiveAddress = address;
            const queryMsg = { balance: { address: injectiveAddress } };

            const response = await fetch(
                `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
                    CONTRACTS.plinkToken
                }/smart/${btoa(JSON.stringify(queryMsg))}`
            );

            const data = await response.json();

            const balance = data?.data?.balance || "0";

            // Convert from base units (18 decimals) to display units
            return new BigNumberInBase(balance)
                .dividedBy(new BigNumberInBase(10).pow(18))
                .toFixed(2);
        } catch (error) {
            console.error("Error fetching PLINK balance:", error);
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
     * Approve PLINK spending for game contract
     */
    async approvePlinkSpending(
        amount: string,
        userAddress: string
    ): Promise<any> {
        try {
            const injectiveAddress = userAddress;

            this.msgBroadcaster = new MsgBroadcaster({
                walletStrategy: this.walletStrategy,
                network,
                endpoints,
                simulateTx: true,
                gasBufferCoefficient: 1.2,
            });

            // Convert amount to base units
            const baseAmount = new BigNumberInBase(amount)
                .times(new BigNumberInBase(10).pow(18))
                .toFixed(0);

            const msg = MsgExecuteContractCompat.fromJSON({
                contractAddress: CONTRACTS.plinkToken,
                sender: injectiveAddress,
                msg: {
                    increase_allowance: {
                        spender: CONTRACTS.game,
                        amount: baseAmount,
                    },
                },
            });

            const result = await this.msgBroadcaster.broadcast({
                msgs: msg,
                injectiveAddress,
            });

            return result;
        } catch (error) {
            console.error("Error approving PLINK spending:", error);
            throw error;
        }
    }

    /**
     * Check PLINK allowance for game contract
     */
    async getPlinkAllowance(userAddress: string): Promise<string> {
        try {
            const injectiveAddress = userAddress;
            const queryMsg = {
                allowance: {
                    owner: injectiveAddress,
                    spender: CONTRACTS.game,
                },
            };

            const response = await fetch(
                `${endpoints.rest}/cosmwasm/wasm/v1/contract/${
                    CONTRACTS.plinkToken
                }/smart/${btoa(JSON.stringify(queryMsg))}`
            );

            const data = await response.json();
            const allowance = data?.data?.allowance || "0";

            return new BigNumberInBase(allowance)
                .dividedBy(new BigNumberInBase(10).pow(18))
                .toFixed(2);
        } catch (error) {
            console.error("Error fetching PLINK allowance:", error);
            return "0.00";
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
        try {
            const injectiveAddress = userAddress;

            this.msgBroadcaster = new MsgBroadcaster({
                walletStrategy: this.walletStrategy,
                network,
                endpoints,
                simulateTx: true,
                gasBufferCoefficient: 1.2,
            });

            const singleBetBaseAmount = new BigNumberInBase(betAmount).times(
                new BigNumberInBase(10).pow(18)
            );

            const totalBetAmount = new BigNumberInBase(betAmount).times(
                numberOfPlays
            );
            const totalBetBaseAmount = singleBetBaseAmount
                .times(numberOfPlays)
                .toFixed(0);

            // This array will hold all messages for our single transaction
            const messages: MsgExecuteContractCompat[] = [];

            // Step 1: Check the current PLINK allowance
            const allowance = await this.getPlinkAllowance(userAddress);

            // Step 2: If allowance is less than the total required, add an approval message
            if (new BigNumberInBase(allowance).lt(totalBetAmount)) {
                const approvalMsg = MsgExecuteContractCompat.fromJSON({
                    contractAddress: CONTRACTS.plinkToken,
                    sender: injectiveAddress,
                    msg: {
                        increase_allowance: {
                            spender: CONTRACTS.game,
                            amount: totalBetBaseAmount, // Approve the total amount
                        },
                    },
                });
                messages.push(approvalMsg);
            }

            // Step 3: Create and add a 'play' message for each ball
            for (let i = 0; i < numberOfPlays; i++) {
                const playMsg = MsgExecuteContractCompat.fromJSON({
                    contractAddress: CONTRACTS.game,
                    sender: injectiveAddress,
                    msg: {
                        play: {
                            difficulty: this.mapDifficulty(difficulty),
                            risk_level: this.mapRiskLevel(riskLevel),
                            bet_amount: singleBetBaseAmount.toFixed(0),
                        },
                    },
                });
                messages.push(playMsg);
            }

            // Step 4: Broadcast the single transaction with all messages
            const result = await this.msgBroadcaster.broadcast({
                msgs: messages,
                injectiveAddress,
            });

            return result;
        } catch (error) {
            console.error("Error playing game:", error);
            throw error;
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
}
