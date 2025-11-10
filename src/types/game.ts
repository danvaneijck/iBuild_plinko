export type Difficulty = "easy" | "medium" | "hard";
export type RiskLevel = "low" | "medium" | "high";

export interface GameSettings {
    difficulty: Difficulty;
    riskLevel: RiskLevel;
    betAmount: string;
}

export interface MultiplierConfig {
    [key: string]: number[];
}

export interface Ball {
    id: string;
    x: number;
    y: number;
    vx: number;
    vy: number;
    currentRow: number;
    pegIndex: number;
    path: number[];
    finalMultiplier: number;
    isActive: boolean;
}

export interface GameResult {
    ballId: string;
    betAmount: string;
    multiplier: string;
    winAmount: string;
    timestamp: number;
}

export type LeaderboardType = "bestWins" | "totalWagered";
