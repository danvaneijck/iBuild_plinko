import React from 'react';
import { History, TrendingUp, TrendingDown } from 'lucide-react';
import { GameResult } from '../types/game';

interface GameHistoryProps {
  history: GameResult[];
}

export const GameHistory: React.FC<GameHistoryProps> = ({ history }) => {
  return (
    <div className="bg-gray-900 rounded-2xl p-6 border border-gray-800 shadow-xl">
      <div className="flex items-center gap-2 mb-6">
        <History className="text-purple-500" size={24} />
        <h2 className="text-2xl font-bold text-white">Recent Games</h2>
      </div>

      <div className="space-y-3 max-h-[400px] overflow-y-auto custom-scrollbar">
        {history.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            No games played yet. Start playing to see your history!
          </div>
        ) : (
          history.map((result) => {
            const isWin = parseFloat(result.winAmount) > parseFloat(result.betAmount);
            const profit = parseFloat(result.winAmount) - parseFloat(result.betAmount);

            return (
              <div
                key={result.ballId}
                className="bg-gray-800 rounded-xl p-4 border border-gray-700 hover:border-gray-600 transition-colors"
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    {isWin ? (
                      <TrendingUp className="text-green-500" size={20} />
                    ) : (
                      <TrendingDown className="text-red-500" size={20} />
                    )}
                    <span className="text-white font-semibold">
                      {result.multiplier}x Multiplier
                    </span>
                  </div>
                  <span className={`font-bold ${isWin ? 'text-green-500' : 'text-red-500'}`}>
                    {profit >= 0 ? '+' : ''}{profit.toFixed(2)} $PLINK
                  </span>
                </div>
                <div className="flex justify-between text-sm text-gray-400">
                  <span>Bet: {result.betAmount} $PLINK</span>
                  <span>Won: {result.winAmount} $PLINK</span>
                </div>
                <div className="text-xs text-gray-600 mt-2">
                  {new Date(result.timestamp).toLocaleString()}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
