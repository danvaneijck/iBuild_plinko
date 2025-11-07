import { useState, useCallback } from "react";
import { Difficulty, RiskLevel, Ball } from "../types/game";
import { useContracts } from "./useContracts";
export const ANIMATION_DURATION_MS = 3000;

export const usePlinkoGame = (userAddress: string) => {
    const [balls, setBalls] = useState<Ball[]>([]);
    const {
        plinkBalance,
        gameHistory,
        isLoading,
        error,
        contractsValid,
        purchasePlink,
        playGame,
        refreshBalance,
        refreshHistory,
    } = useContracts(userAddress);

    const dropBall = useCallback(
        async (
            difficulty: Difficulty,
            riskLevel: RiskLevel,
            betAmount: string
        ) => {
            if (!contractsValid) {
                throw new Error("Contracts not configured.");
            }

            try {
                const gameResult = await playGame(
                    difficulty,
                    riskLevel,
                    betAmount
                );

                if (gameResult && gameResult.path) {
                    const newBall: Ball = {
                        id: `ball-${Date.now()}`,
                        path: gameResult.path,
                    };
                    setBalls((prev) => [...prev, newBall]);

                    setTimeout(() => {
                        console.log(
                            "Animation finished. Updating balance and history."
                        );
                        refreshBalance();
                        refreshHistory();

                        setBalls((prev) =>
                            prev.filter((b) => b.id !== newBall.id)
                        );
                    }, ANIMATION_DURATION_MS);
                }
                return gameResult;
            } catch (err: any) {
                console.error("Drop ball failed:", err);
                throw err;
            }
        },
        [playGame, contractsValid, refreshBalance, refreshHistory]
    );

    const handlePurchasePlink = useCallback(
        async (injAmount: string) => {
            if (!contractsValid) {
                throw new Error(
                    "Contracts not configured. Please deploy contracts first."
                );
            }

            await purchasePlink(injAmount);
        },
        [purchasePlink, contractsValid]
    );

    return {
        balls,
        gameHistory,
        plinkBalance,
        isLoading,
        error,
        contractsValid,
        dropBall,
        purchasePlink: handlePurchasePlink,
        refreshBalance,
    };
};
