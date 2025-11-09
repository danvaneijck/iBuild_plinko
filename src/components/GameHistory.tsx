import React from 'react';
import { History, TrendingUp, TrendingDown } from 'lucide-react';
import { GameResult } from '../types/game';
import { formatTokenAmount } from '../utils/format';
import { PLINK_TOKEN_DECIMALS } from '../config/contracts';
import { motion, AnimatePresence } from 'framer-motion'; // Import Framer Motion

interface GameHistoryProps {
  history: GameResult[];
}

export const GameHistory: React.FC<GameHistoryProps> = ({ history }) => {
  return (
    <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-4 space-y-3">
      <div className="flex items-center gap-2 mb-6">
        <History className="text-purple-500" size={24} />
        <h2 className="text-2xl font-bold text-white">Your Games</h2>
      </div>

      <div className="space-y-2 max-h-[400px] overflow-y-auto custom-scrollbar">
        {history.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            No games played yet. Start playing to see your history!
          </div>
        ) : (

          <AnimatePresence initial={false}>
            {history.map((result, i) => {
              const isWin = BigInt(result.win_amount || '0') > BigInt(result.bet_amount || '0');
              const profitBigInt = BigInt(result.win_amount || '0') - BigInt(result.bet_amount || '0');
              const betFormatted = formatTokenAmount(result.bet_amount, PLINK_TOKEN_DECIMALS);
              const wonFormatted = formatTokenAmount(result.win_amount, PLINK_TOKEN_DECIMALS);
              const profitFormatted = formatTokenAmount(
                profitBigInt.toString().replace('-', ''),
                PLINK_TOKEN_DECIMALS
              );

              return (
                <motion.div
                  key={result.ballId || result.timestamp + i} // Use a stable unique key like ballId
                  layout
                  initial={{ opacity: 0, y: -20, scale: 0.95 }} // Start invisible and slightly above
                  animate={{ opacity: 1, y: 0, scale: 1 }} // Animate to fully visible and in place
                  transition={{ duration: 0.1, ease: "easeInOut" }} // Smooth transition
                  className="bg-gray-900/50 rounded-xl p-4 "
                >
                  <div className="flex items-center justify-between mb-1">
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
                </motion.div>
              );
            })}
          </AnimatePresence>
        )}
      </div>
    </div>
  );
};