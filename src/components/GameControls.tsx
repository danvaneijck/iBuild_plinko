import React, { useState } from 'react';
import { Difficulty, RiskLevel } from '../types/game';

interface GameControlsProps {
  onPlay: (betAmount: string, numberOfBalls: number) => void; // Updated prop
  plinkBalance: string;
  disabled: boolean;
  isLoading: boolean;
  difficulty: Difficulty;
  riskLevel: RiskLevel;
  onDifficultyChange: (difficulty: Difficulty) => void;
  onRiskLevelChange: (riskLevel: RiskLevel) => void;
}

export const GameControls: React.FC<GameControlsProps> = ({
  onPlay,
  plinkBalance,
  disabled,
  isLoading,
  difficulty,
  riskLevel,
  onDifficultyChange,
  onRiskLevelChange,
}) => {
  const [betAmount, setBetAmount] = useState('10');
  const [numberOfBalls, setNumberOfBalls] = useState(1);

  const handlePlayClick = () => {

    if (!disabled && !isLoading) {
      onPlay(betAmount, numberOfBalls); // Call onPlay once
    }
  };

  const difficulties: Difficulty[] = ['easy', 'medium', 'hard'];
  const riskLevels: RiskLevel[] = ['low', 'medium', 'high'];

  const getButtonClass = (isActive: boolean) =>
    `px-4 py-2 rounded-lg font-semibold transition-all duration-200 ${isActive
      ? 'bg-purple-600 text-white shadow-md'
      : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
    }`;

  return (
    <div className="bg-gray-800/50 border border-gray-700 rounded-2xl p-6 space-y-6 shadow-xl fade-in">
      <div>
        <label className="text-sm font-medium text-gray-400">Bet Amount</label>
        <div className="mt-2 flex">
          <input
            type="number"
            value={betAmount}
            onChange={(e) => setBetAmount(e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded-l-lg p-3 text-white focus:ring-2 focus:ring-purple-500 focus:outline-none"
            placeholder="Enter amount"
            disabled={disabled || isLoading}
          />
          <button
            onClick={() => setBetAmount(String(parseFloat(betAmount) / 2))}
            className="bg-gray-700 px-4 hover:bg-gray-600 text-white"
            disabled={disabled || isLoading}
          >
            ½
          </button>
          <button
            onClick={() => setBetAmount(String(parseFloat(betAmount) * 2))}
            className="bg-gray-700 px-4 hover:bg-gray-600 text-white rounded-r-lg"
            disabled={disabled || isLoading}
          >
            2x
          </button>
        </div>
        <p className="text-xs text-gray-400 mt-2">
          Your balance: <span className="font-bold text-purple-400">{plinkBalance} $PLINK</span>
        </p>
      </div>

      <div>
        <label className="text-sm font-medium text-gray-400">Number of Balls</label>
        <div className="mt-2 flex">
          <input
            type="number"
            value={numberOfBalls}
            onChange={(e) => setNumberOfBalls(parseInt(e.target.value, 10))}
            className="w-full bg-gray-900 border border-gray-700 rounded-lg p-3 text-white focus:ring-2 focus:ring-purple-500 focus:outline-none"
            placeholder="Enter number of balls"
            min={1}
            max={20}
            disabled={disabled || isLoading}
          />
        </div>
      </div>

      <div>
        <label className="text-sm font-medium text-gray-400">Difficulty (Rows)</label>
        <div className="grid grid-cols-3 gap-2 mt-2">
          {difficulties.map((d) => (
            <button key={d} onClick={() => onDifficultyChange(d)} className={getButtonClass(difficulty === d)}>
              {d.charAt(0).toUpperCase() + d.slice(1)}
            </button>
          ))}
        </div>
      </div>

      <div>
        <label className="text-sm font-medium text-gray-400">Risk Level</label>
        <div className="grid grid-cols-3 gap-2 mt-2">
          {riskLevels.map((r) => (
            <button key={r} onClick={() => onRiskLevelChange(r)} className={getButtonClass(riskLevel === r)}>
              {r.charAt(0).toUpperCase() + r.slice(1)}
            </button>
          ))}
        </div>
      </div>

      <button
        onClick={handlePlayClick}
        disabled={disabled || isLoading || parseFloat(betAmount) <= 0}
        className="w-full py-4 bg-gradient-to-r from-green-500 to-emerald-600 text-white font-bold text-lg rounded-xl transition-all duration-300 shadow-lg hover:shadow-2xl disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isLoading ? 'Processing...' : 'Play'}
      </button>
    </div>
  );
};