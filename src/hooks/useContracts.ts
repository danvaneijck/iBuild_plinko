import { useState, useEffect, useCallback } from "react";
import { WalletStrategy } from "@injectivelabs/wallet-strategy";
import { ChainId, EvmChainId } from "@injectivelabs/ts-types";
import { ContractService } from "../services/contractService";
import { CHAIN_ID, validateContracts } from "../config/contracts";
import { Difficulty, RiskLevel, GameResult } from "../types/game";

const chainId = CHAIN_ID === "injective-1" ? ChainId.Mainnet : ChainId.Testnet;

export const walletStrategy = new WalletStrategy({
    chainId,
    evmOptions: {
        rpcUrl: "https://sentry.evm-rpc.injective.network/",
        evmChainId: EvmChainId.TestnetEvm,
    },
    strategies: {},
});

export const useContracts = (userAddress: string) => {
    const [plinkBalance, setPlinkBalance] = useState<string>("0.00");
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string>("");
    const [gameHistory, setGameHistory] = useState<any[]>([]);
    const [contractsValid, setContractsValid] = useState(false);

    // Validate contracts on mount
    useEffect(() => {
        const valid = validateContracts();
        setContractsValid(valid);
        if (!valid) {
            setError(
                "Contract addresses not configured. Please deploy contracts first."
            );
        }
    }, []);

    // Fetch PLINK balance
    const fetchPlinkBalance = useCallback(async () => {
        if (!userAddress || !contractsValid) return;

        const contractService = new ContractService(walletStrategy);

        try {
            const balance = await contractService.getPlinkBalance(userAddress);
            if (balance) setPlinkBalance(balance);
        } catch (err: any) {
            console.error("Error fetching balance:", err);
            setError(err.message || "Failed to fetch balance");
        }
    }, [userAddress, contractsValid]);

    // Fetch game history
    const fetchGameHistory = useCallback(async () => {
        if (!userAddress || !contractsValid) return;
        const contractService = new ContractService(walletStrategy);
        try {
            const history = await contractService.getGameHistory(
                userAddress,
                20
            );
            setGameHistory(history);
        } catch (err: any) {
            console.error("Error fetching history:", err);
        }
    }, [userAddress, contractsValid]);

    const getGameHistory = useCallback(
        async (limit: number = 20) => {
            const contractService = new ContractService(walletStrategy);

            if (!userAddress || !contractsValid) return [];
            try {
                const historyFromContract =
                    await contractService.getGameHistory(userAddress, limit);
                return historyFromContract.map((game, index) => ({
                    ...game,

                    ballId: game.ballId || `hist-${game.timestamp}-${index}`,
                }));
            } catch (err: any) {
                console.error("Error fetching history:", err);
                return [];
            }
        },
        [userAddress, contractsValid]
    );

    // Auto-fetch on address change
    useEffect(() => {
        if (userAddress && contractsValid) {
            fetchPlinkBalance();
            fetchGameHistory();
        }
    }, [userAddress, contractsValid, fetchPlinkBalance, fetchGameHistory]);

    // Purchase PLINK
    const purchasePlink = useCallback(
        async (injAmount: string) => {
            const contractService = new ContractService(walletStrategy);
            if (!userAddress || !contractsValid) {
                throw new Error(
                    "Wallet not connected or contracts not configured"
                );
            }

            setIsLoading(true);
            setError("");

            try {
                const result = await contractService.purchasePlink(
                    injAmount,
                    userAddress
                );

                // Refresh balance after purchase
                await fetchPlinkBalance();

                return result;
            } catch (err: any) {
                console.error("Purchase error:", err);
                setError(err.message || "Failed to purchase PLINK");
                throw err;
            } finally {
                setIsLoading(false);
            }
        },
        [userAddress, fetchPlinkBalance, contractsValid]
    );

    // Play game
    const playGame = useCallback(
        async (
            difficulty: Difficulty,
            riskLevel: RiskLevel,
            betAmount: string,
            numberOfBalls: number // New parameter
        ) => {
            const contractService = new ContractService(walletStrategy);
            if (!userAddress || !contractsValid) {
                setError("Wallet not connected or contracts not configured");

                throw new Error(
                    "Wallet not connected or contracts not configured"
                );
            }

            if (numberOfBalls < 1 || numberOfBalls > 50) {
                setError("Number of balls must be between 1 and 50");
                return;
            }

            if (parseFloat(betAmount) > parseFloat(plinkBalance)) {
                setError("Insufficient PLINK balance");
                throw new Error("Insufficient PLINK balance");
            }

            if (
                parseFloat(betAmount) * numberOfBalls >
                parseFloat(plinkBalance)
            ) {
                setError("Insufficient PLINK balance");
                throw new Error("Insufficient PLINK balance");
            }

            setIsLoading(true);
            setError("");

            try {
                const result = await contractService.playGame(
                    difficulty,
                    riskLevel,
                    betAmount,
                    numberOfBalls,
                    userAddress
                );

                const gameResult = parseMultipleGameResults(result);

                return gameResult;
            } catch (err: any) {
                console.error("Play game error:", err);
                setError(err.message || "Failed to play game");
                throw err;
            } finally {
                setIsLoading(false);
            }
        },
        [userAddress, plinkBalance, contractsValid]
    );

    const parseMultipleGameResults = (txResult: any): GameResult[] => {
        try {
            const events = txResult?.events || [];
            const wasmEvents = events.filter(
                (e: any) =>
                    e.type === "wasm" &&
                    e.attributes.some(
                        (a: any) => a.key === "action" && a.value === "play"
                    )
            );

            if (wasmEvents.length === 0) return [];

            return wasmEvents.map((event: any, i) => {
                const attrs = event.attributes || [];
                const getAttr = (key: string) =>
                    attrs.find((a: any) => a.key === key)?.value;

                const pathString = getAttr("path");
                // Use a more robust unique ID
                const uniqueId = `ball-${Date.now()}-${Math.random()
                    .toString(36)
                    .substring(2, 9)}`;

                return {
                    ballId: uniqueId,
                    bet_amount: getAttr("bet_amount"),
                    multiplier: `${parseFloat(getAttr("multiplier"))}x`,
                    win_amount: getAttr("win_amount"),
                    timestamp: Date.parse(txResult.timestamp) / 1000,
                    path: pathString ? pathString.split("").map(Number) : [],
                    eventIndex: i,
                };
            });
        } catch (err) {
            console.error("Error parsing game results:", err);
            return [];
        }
    };

    // Parse game result from transaction
    const parseGameResult = (txResult: any): GameResult | null => {
        try {
            console.log("Transaction result:", txResult);
            // Extract attributes from transaction events
            const events = txResult?.events || [];
            const wasmEvent = events.find(
                (e: any) =>
                    e.type === "wasm" &&
                    e.attributes.some(
                        (a) => a.key == "action" && a.value == "play"
                    )
            );

            if (!wasmEvent) return null;

            const attrs = wasmEvent.attributes || [];
            const getAttr = (key: string) =>
                attrs.find((a: any) => a.key === key)?.value;

            const betAmount = getAttr("bet_amount");
            const winAmount = getAttr("win_amount");
            const multiplier = getAttr("multiplier");
            const pathString = getAttr("path");

            if (!betAmount || !winAmount || !multiplier || !pathString) {
                console.error(
                    "Essential game attributes missing from transaction event"
                );
                return null;
            }

            const path = pathString.split("").map(Number);

            return {
                ballId: `ball-${Date.now()}`,
                betAmount,
                multiplier: parseFloat(multiplier),
                winAmount,
                timestamp: Date.now(),
                path: path,
            };
        } catch (err) {
            console.error("Error parsing game result:", err);
            return null;
        }
    };

    return {
        plinkBalance,
        gameHistory,
        isLoading,
        error,
        contractsValid,
        purchasePlink,
        playGame,
        refreshBalance: fetchPlinkBalance,
        getGameHistory,
    };
};
