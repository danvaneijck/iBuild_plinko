import React, { useState } from 'react';
import { Play, Settings, Loader } from 'lucide-react';
import { Difficulty, RiskLevel } from '../types/game';

interface GameControlsProps {
  onPlay: (difficulty: Difficulty, riskLevel: RiskLevel, betAmount: string) => Promise<void>;
  plinkBalance: string;
  disabled?: boolean;
  isLoading?: boolean;
}

export const GameControls: React.FC<GameControlsProps> = ({ 
  onPlay, 
  plinkBalance, 
  disabled,
  isLoading = false 
}) => {
  const [difficulty, setDifficulty] = useState<Difficulty>('medium');
  const [riskLevel, setRiskLevel] = useState<RiskLevel>('medium');
  const [betAmount, setBetAmount] = useState('10');
  const [error, setError] = useState('');

  const handlePlay = async () => {
    if (parseFloat(betAmount) > parseFloat(plinkBalance)) {
      setError('Insufficient $PLINK balance!');
      return;
    }
    if (parseFloat(betAmount) <= 0) {
      setError('Bet amount must be greater than 0!');
      return;
    }

    try {
      setError('');
      await onPlay(difficulty, riskLevel, betAmount);
    } catch (err: any) {
      setError(err.message || 'Failed to play game');
    }
  };

  return (
    <div className="bg-gray-900 rounded-2xl p-6 border border-gray-800 shadow-xl">
      <div className="flex items-center gap-2 mb-6">
        <Settings className="text-purple-500" size={24} />
        <h2 className="text-2xl font-bold text-white">Game Settings</h2>
      </div>

      <div className="space-y-6">
        {/* Difficulty */}
        <div>
          <label className="block text-sm font-semibold text-gray-400 mb-3">Difficulty</label>
          <div className="grid grid-cols-3 gap-3">
            {(['easy', 'medium', 'hard'] as Difficulty[]).map((diff) => (
              <button
                key={diff}
                onClick={() => setDifficulty(diff)}
                disabled={disabled || isLoading}
                className={`py-3 px-4 rounded-xl font-semibold transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed ${
                  difficulty === diff
                    ? 'bg-gradient-to-r from-purple-600 to-pink-600 text-white shadow-lg'
                    : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                }`}
              >
                {diff.charAt(0).toUpperCase() + diff.slice(1)}
                <div className="text-xs mt-1 opacity-75">
                  {diff === 'easy' ? '8 rows' : diff === 'medium' ? '12 rows' : '16 rows'}
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Risk Level */}
        <div>
          <label className="block text-sm font-semibold text-gray-400 mb-3">Risk Level</label>
          <div className="grid grid-cols-3 gap-3">
            {(['low', 'medium', 'high'] as RiskLevel[]).map((risk) => (
              <button
                key={risk}
                onClick={() => setRiskLevel(risk)}
                disabled={disabled || isLoading}
                className={`py-3 px-4 rounded-xl font-semibold transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed ${
                  riskLevel === risk
                    ? 'bg-gradient-to-r from-purple-600 to-pink-600 text-white shadow-lg'
                    : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                }`}
              >
                {risk.charAt(0).toUpperCase() + risk.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {/* Bet Amount */}
        <div>
          <label className="block text-sm font-semibold text-gray-400 mb-3">Bet Amount ($PLINK)</label>
          <div className="relative">
            <input
              type="number"
              value={betAmount}
              onChange={(e) => {
                setBetAmount(e.target.value);
                setError('');
              }}
              disabled={disabled || isLoading}
              className="w-full bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 text-white font-semibold focus:outline-none focus:border-purple-500 transition-colors disabled:opacity-50"
              placeholder="Enter bet amount"
              min="0"
              step="0.01"
            />
            <div className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 text-sm">
              Balance: {plinkBalance}
            </div>
          </div>
        </div>

        {error && (
          <div className="bg-red-900/20 border border-red-700 rounded-lg p-3">
            <p className="text-sm text-red-400">{error}</p>
          </div>
        )}

        {/* Play Button */}
        <button
          onClick={handlePlay}
          disabled={disabled || isLoading}
          className="w-full py-4 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white font-bold rounded-xl transition-all duration-200 shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {isLoading ? (
            <>
              <Loader className="animate-spin" size={20} />
              Processing...
            </>
          ) : (
            <>
              <Play size={20} fill="currentColor" />
              Drop Ball
            </>
          )}
        </button>
      </div>
    </div>
  );
};
