import React, { useState } from 'react';
import { X, Coins, Loader } from 'lucide-react';
import { EXCHANGE_RATE } from '../config/contracts';

interface PurchaseModalProps {
  isOpen: boolean;
  onClose: () => void;
  onPurchase: (injAmount: string) => Promise<void>;
  isLoading?: boolean;
}

export const PurchaseModal: React.FC<PurchaseModalProps> = ({
  isOpen,
  onClose,
  onPurchase,
  isLoading = false
}) => {
  const [injAmount, setInjAmount] = useState('1');
  const [error, setError] = useState('');

  if (!isOpen) return null;

  const handlePurchase = async () => {
    if (parseFloat(injAmount) <= 0) {
      setError('Amount must be greater than 0!');
      return;
    }

    try {
      setError('');
      await onPurchase(injAmount);
      onClose();
      setInjAmount('1');
    } catch (err: any) {
      setError(err.message || 'Failed to purchase PLINK');
    }
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-gray-900 rounded-2xl p-8 max-w-md w-full border border-gray-800 shadow-2xl animate-fade-in-scale">
        <div className="flex justify-between items-center mb-6">
          <div className="flex items-center gap-2">
            <Coins className="text-purple-500" size={28} />
            <h2 className="text-2xl font-bold text-white">Purchase $PLINK</h2>
          </div>
          <button
            onClick={onClose}
            disabled={isLoading}
            className="p-2 hover:bg-gray-800 rounded-lg transition-colors disabled:opacity-50"
          >
            <X size={24} className="text-gray-400" />
          </button>
        </div>

        <div className="space-y-6">
          <div className="bg-gray-800 rounded-xl p-4 border border-gray-700">
            <div className="text-sm text-gray-400 mb-2">Exchange Rate</div>
            <div className="text-2xl font-bold text-white">1 INJ = {EXCHANGE_RATE} $PLINK</div>
          </div>

          <div>
            <label className="block text-sm font-semibold text-gray-400 mb-3">INJ Amount</label>
            <input
              type="number"
              value={injAmount}
              onChange={(e) => {
                setInjAmount(e.target.value);
                setError('');
              }}
              disabled={isLoading}
              className="w-full bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 text-white font-semibold focus:outline-none focus:border-purple-500 transition-colors disabled:opacity-50"
              placeholder="Enter INJ amount"
              min="0"
              step="0.01"
            />
          </div>

          <div className="bg-gradient-to-r from-purple-900/50 to-pink-900/50 rounded-xl p-4 border border-purple-700/50">
            <div className="text-sm text-gray-400 mb-1">You will receive</div>
            <div className="text-3xl font-bold text-white">
              {(parseFloat(injAmount || '0') * EXCHANGE_RATE).toFixed(2)} $PLINK
            </div>
          </div>

          {error && (
            <div className="bg-red-900/20 border border-red-700 rounded-lg p-3">
              <p className="text-sm text-red-400">{error}</p>
            </div>
          )}

          <button
            onClick={handlePurchase}
            disabled={isLoading}
            className="w-full py-4 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white font-bold rounded-xl transition-all duration-200 shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          >
            {isLoading ? (
              <>
                <Loader className="animate-spin" size={20} />
                Processing...
              </>
            ) : (
              'Purchase $PLINK'
            )}
          </button>

          <div className="text-xs text-gray-500 text-center">
            All INJ will be sent to the treasury wallet
          </div>
        </div>
      </div>
    </div>
  );
};
