import { useState, useCallback, useEffect } from "react";
import { Difficulty, RiskLevel, Ball, GameResult } from "../types/game";
import { useContracts } from "./useContracts";

const CANVAS_WIDTH = 800;
const SPACING = 45;
const BALL_DROP_DELAY_MS = 500; // 200ms delay between ball drops

export const usePlinkoGame = (userAddress: string) => {
    const [balls, setBalls] = useState<Ball[]>([]);
    const [gameHistory, setGameHistory] = useState<GameResult[]>([]);
    const [pendingResults, setPendingResults] = useState<GameResult[]>([]);

    const {
        plinkBalance,
        isLoading,
        error,
        contractsValid,
        purchasePlink,
        playGame,
        refreshBalance,
        getGameHistory,
    } = useContracts(userAddress);

    useEffect(() => {
        const fetchInitialHistory = async () => {
            if (userAddress && contractsValid) {
                const history = await getGameHistory(20);
                setGameHistory(history);
            }
        };
        fetchInitialHistory();
    }, [userAddress, contractsValid, getGameHistory]);

    const handleAnimationComplete = useCallback(
        (ballId: string) => {
            // Find the result for the ball that just finished animating
            const finishedResult = pendingResults.find(
                (p) => p.ballId === ballId
            );

            if (finishedResult) {
                // Add the finished result to the top of the visible history list
                setGameHistory((prev) => [finishedResult, ...prev]);
                // Remove it from the pending queue
                setPendingResults((prev) =>
                    prev.filter((p) => p.ballId !== ballId)
                );
            }

            // Refresh balance now that the outcome is settled
            if (finishedResult && pendingResults.length === 1) refreshBalance();

            // Remove the ball from the animation board
            setBalls((prev) => prev.filter((b) => b.id !== ballId));
        },
        [pendingResults, refreshBalance]
    );

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
                    setPendingResults((prev) => [...prev, ...gameResults]);

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
                                eventIndex: result.eventIndex,
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
