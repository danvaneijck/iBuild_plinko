import { useState, useCallback } from 'react';
import { Difficulty, RiskLevel, Ball, GameResult } from '../types/game';
import { MULTIPLIERS, ROWS_CONFIG } from '../config/multipliers';
import { useContracts } from './useContracts';

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
  } = useContracts(userAddress);

  const generateBallPath = useCallback((difficulty: Difficulty): number[] => {
    const rows = ROWS_CONFIG[difficulty];
    const path: number[] = [];
    
    for (let i = 0; i < rows; i++) {
      path.push(Math.random() > 0.5 ? 1 : 0);
    }
    
    return path;
  }, []);

  const calculateFinalPosition = useCallback((path: number[]): number => {
    return path.reduce((sum, direction) => sum + direction, 0);
  }, []);

  const dropBall = useCallback(
    async (difficulty: Difficulty, riskLevel: RiskLevel, betAmount: string) => {
      if (!contractsValid) {
        throw new Error('Contracts not configured. Please deploy contracts first.');
      }

      try {
        // Start visual animation immediately
        const path = generateBallPath(difficulty);
        const finalPosition = calculateFinalPosition(path);
        const multipliers = MULTIPLIERS[difficulty][riskLevel];
        const finalMultiplier = multipliers[finalPosition] || 0;

        const newBall: Ball = {
          id: `ball-${Date.now()}-${Math.random()}`,
          x: 0,
          y: 0,
          vx: 0,
          vy: 0,
          path,
          finalMultiplier,
          isActive: true,
        };

        setBalls((prev) => [...prev, newBall]);

        // Execute blockchain transaction
        const result = await playGame(difficulty, riskLevel, betAmount);

        // Update ball with actual result after transaction
        setTimeout(() => {
          setBalls((prev) => prev.filter((b) => b.id !== newBall.id));
        }, 3000);

        return result;
      } catch (err: any) {
        // Remove ball on error
        setBalls((prev) => prev.filter((b) => b.id !== newBall.id));
        throw err;
      }
    },
    [generateBallPath, calculateFinalPosition, playGame, contractsValid]
  );

  const handlePurchasePlink = useCallback(
    async (injAmount: string) => {
      if (!contractsValid) {
        throw new Error('Contracts not configured. Please deploy contracts first.');
      }

      try {
        await purchasePlink(injAmount);
      } catch (err) {
        throw err;
      }
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
