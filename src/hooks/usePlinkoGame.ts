import { useState, useCallback, useEffect, useRef } from "react";
import { Difficulty, RiskLevel, Ball, GameResult } from "../types/game";
import { useContracts } from "./useContracts";

const CANVAS_WIDTH = 800;
const SPACING = 45;
const MIN_DELAY_MS = 200; // The fastest possible time between ball drops
const MAX_DELAY_MS = 500; // The slowest possible time between ball drops

export const usePlinkoGame = (
    userAddress: string,
    difficulty: Difficulty,
    riskLevel: RiskLevel
) => {
    const [balls, setBalls] = useState<Ball[]>([]);
    const [gameHistory, setGameHistory] = useState<GameResult[]>([]);
    const [pendingResults, setPendingResults] = useState<GameResult[]>([]);
    const pendingDropTimeoutsRef = useRef<NodeJS.Timeout[]>([]);

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
        setBalls([]);
        setPendingResults([]);
        pendingDropTimeoutsRef.current.forEach(clearTimeout);
        pendingDropTimeoutsRef.current = [];
        refreshBalance();
    }, [difficulty, riskLevel, refreshBalance]);

    useEffect(() => {
        const fetchInitialHistory = async () => {
            if (userAddress && contractsValid) {
                const history = await getGameHistory(20);
                const sortedHistory = history.sort(
                    (a, b) => b.timestamp - a.timestamp
                );
                setGameHistory(sortedHistory);
            }
        };
        fetchInitialHistory();
    }, [userAddress, contractsValid, getGameHistory]);

    const handleAnimationComplete = useCallback(
        (ballId: string) => {
            let finishedResult: GameResult | undefined;

            // Use the functional update form to get the most recent state
            setPendingResults((currentPendingResults) => {
                finishedResult = currentPendingResults.find(
                    (p) => p.ballId === ballId
                );

                if (currentPendingResults.length === 1 && finishedResult) {
                    refreshBalance();
                }

                return currentPendingResults.filter((p) => p.ballId !== ballId);
            });

            const result = pendingResults.find((p) => p.ballId === ballId);
            if (result) {
                setGameHistory((prev) => [result, ...prev]);
            }

            setPendingResults((currentPending) => {
                if (currentPending.length === 1) {
                    refreshBalance();
                }
                return currentPending.filter((p) => p.ballId !== ballId);
            });

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
            numberOfBalls: number
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

                    // This variable will track the total delay for the next ball.
                    let cumulativeDelay = 0;

                    // Loop through results and add each ball to the state with a random delay
                    gameResults.forEach((result) => {
                        // Schedule the creation of the current ball using the current cumulative delay.
                        // For the first ball, this will be 0, so it drops instantly.
                        const timeoutId = setTimeout(() => {
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
                        }, cumulativeDelay);

                        pendingDropTimeoutsRef.current.push(timeoutId);

                        // Then, calculate a random interval for the *next* ball and add it to the total.
                        const randomInterval =
                            MIN_DELAY_MS +
                            Math.random() * (MAX_DELAY_MS - MIN_DELAY_MS);
                        cumulativeDelay += randomInterval;
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
