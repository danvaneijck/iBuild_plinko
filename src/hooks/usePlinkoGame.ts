import { useState, useCallback } from "react";
import { Difficulty, RiskLevel, Ball } from "../types/game";
import { useContracts } from "./useContracts";
import { ROWS_CONFIG } from "../config/multipliers";
export const ANIMATION_DURATION_MS = 3000;

const CANVAS_WIDTH = 800;
const SPACING = 45;

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

    const handleAnimationComplete = useCallback(
        (ballId: string) => {
            console.log(`Animation for ${ballId} complete. Refreshing data.`);

            // Refresh the user's balance and game history.
            refreshBalance();
            refreshHistory();

            // Remove the completed ball from the state.
            setBalls((prev) => prev.filter((b) => b.id !== ballId));
        },
        [refreshBalance, refreshHistory]
    ); // Dependencies

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
                    // Calculate starting position based on first row center
                    const rows = ROWS_CONFIG[difficulty];
                    const firstRowPegs = 3; // First row always has 3 pegs
                    const firstRowWidth = (firstRowPegs - 1) * SPACING;
                    const firstRowStartX = (CANVAS_WIDTH - firstRowWidth) / 2;
                    const centerPegIndex = Math.floor(firstRowPegs / 2); // Middle peg (index 1 for 3 pegs)
                    const startX = firstRowStartX + centerPegIndex * SPACING;

                    const newBall: Ball = {
                        id: `ball-${Date.now()}`,
                        path: gameResult.path,
                        x: startX, // Start aligned with center peg of first row
                        y: 40, // Start above the pegs
                        vx: 0,
                        vy: 0,
                        currentRow: -1,
                        pegIndex: centerPegIndex,
                    };
                    setBalls((prev) => [...prev, newBall]);
                }
                return gameResult;
            } catch (err: any) {
                console.error("Drop ball failed:", err);
                throw err;
            }
        },
        [playGame, contractsValid]
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
        onAnimationComplete: handleAnimationComplete,
    };
};
