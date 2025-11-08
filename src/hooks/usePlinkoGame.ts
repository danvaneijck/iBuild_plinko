import { useState, useCallback } from "react";
import { Difficulty, RiskLevel, Ball } from "../types/game";
import { useContracts } from "./useContracts";

const CANVAS_WIDTH = 800;
const SPACING = 45;
const BALL_DROP_DELAY_MS = 200; // 200ms delay between ball drops

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
            betAmount: string,
            numberOfBalls: number // New parameter
        ) => {
            if (!contractsValid) {
                throw new Error("Contracts not configured.");
            }

            try {
                const gameResults = await playGame(
                    difficulty,
                    riskLevel,
                    betAmount,
                    numberOfBalls
                );

                if (gameResults && gameResults.length > 0) {
                    const firstRowPegs = 3;
                    const firstRowWidth = (firstRowPegs - 1) * SPACING;
                    const firstRowStartX = (CANVAS_WIDTH - firstRowWidth) / 2;
                    const centerPegIndex = Math.floor(firstRowPegs / 2);
                    const startX = firstRowStartX + centerPegIndex * SPACING;

                    // Loop through results and add each ball to the state with a delay
                    gameResults.forEach((result, index) => {
                        setTimeout(() => {
                            const newBall: Ball = {
                                id: result.ballId,
                                path: result.path,
                                x: startX,
                                y: 40,
                                vx: 0,
                                vy: 0,
                                currentRow: -1,
                                pegIndex: centerPegIndex,
                            };
                            setBalls((prev) => [...prev, newBall]);
                        }, index * BALL_DROP_DELAY_MS); // Stagger the animation
                    });
                }
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
