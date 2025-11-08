import React from 'react';
import { History, TrendingUp, TrendingDown } from 'lucide-react';
import { GameResult } from '../types/game';
import { formatTokenAmount } from '../utils/format';
import { PLINK_TOKEN_DECIMALS } from '../config/contracts';

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
          history.sort((a, b) => b.timestamp - a.timestamp).map((result, i) => {

            // ---- Step 2: Use BigInt for precise calculations ----
            const betAmountBigInt = BigInt(result.bet_amount || '0');
            const winAmountBigInt = BigInt(result.win_amount || '0');
            const profitBigInt = winAmountBigInt - betAmountBigInt;
            const isWin = winAmountBigInt > betAmountBigInt;

            // ---- Step 3: Format the amounts for display ----
            const betFormatted = formatTokenAmount(result.bet_amount, PLINK_TOKEN_DECIMALS);
            const wonFormatted = formatTokenAmount(result.win_amount, PLINK_TOKEN_DECIMALS);
            // We format the profit separately. Need to handle its sign.
            const profitFormatted = formatTokenAmount(
              profitBigInt.toString().replace('-', ''), // Format the absolute value
              PLINK_TOKEN_DECIMALS
            );

            return (
              <div
                key={result.timestamp + i}
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
                      {result.multiplier} Multiplier
                    </span>
                  </div>
                  <span className={`font-bold ${isWin ? 'text-green-500' : 'text-red-500'}`}>
                    {profitBigInt >= 0n ? '+' : '-'}{profitFormatted} $PLINK
                  </span>
                </div>
                <div className="flex justify-between text-sm text-gray-400">
                  <span>Bet: {betFormatted} $PLINK</span>
                  <span>Won: {wonFormatted} $PLINK</span>
                </div>
                <div className="text-xs text-gray-600 mt-2">
                  {new Date(result.timestamp * 1000).toLocaleString()}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};