import React, { useEffect, useState } from 'react';
import { AlertCircle, CheckCircle, ExternalLink } from 'lucide-react';
import { CONTRACTS, validateContracts, NETWORK } from '../config/contracts';

export const ContractStatus: React.FC = () => {
  const [isValid, setIsValid] = useState(false);
  const [showDetails, setShowDetails] = useState(false);

  useEffect(() => {
    setIsValid(validateContracts());
  }, []);

  const explorerUrl = NETWORK === 'mainnet' 
    ? 'https://explorer.injective.network/contract'
    : 'https://testnet.explorer.injective.network/contract';

  if (isValid && !showDetails) {
    return (
      <div className="fixed bottom-4 right-4 z-50">
        <button
          onClick={() => setShowDetails(true)}
          className="flex items-center gap-2 bg-green-900/50 border border-green-700 rounded-lg px-4 py-2 text-green-400 hover:bg-green-900/70 transition-colors"
        >
          <CheckCircle size={16} />
          <span className="text-sm font-semibold">Contracts Active</span>
        </button>
      </div>
    );
  }

  if (!isValid) {
    return (
      <div className="fixed inset-0 bg-black/90 backdrop-blur-sm flex items-center justify-center z-50 p-4">
        <div className="bg-gray-900 rounded-2xl p-8 max-w-2xl w-full border border-red-700 shadow-2xl">
          <div className="flex items-center gap-3 mb-6">
            <AlertCircle className="text-red-500" size={32} />
            <h2 className="text-2xl font-bold text-white">Contracts Not Configured</h2>
          </div>

          <div className="space-y-4 mb-6">
            <p className="text-gray-300">
              The smart contracts have not been deployed yet. Please follow these steps:
            </p>

            <ol className="list-decimal list-inside space-y-2 text-gray-400">
              <li>Build the contracts on your local machine (requires Rust)</li>
              <li>Deploy contracts to Injective testnet</li>
              <li>Update the <code className="bg-gray-800 px-2 py-1 rounded">.env</code> file with contract addresses</li>
              <li>Restart the development server</li>
            </ol>

            <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
              <p className="text-sm text-gray-400 mb-2">See deployment instructions:</p>
              <code className="text-xs text-purple-400">contracts/README.md</code>
            </div>
          </div>

          <div className="bg-yellow-900/20 border border-yellow-700 rounded-lg p-4">
            <p className="text-sm text-yellow-400">
              <strong>Note:</strong> Building contracts requires a native Rust toolchain and cannot be done in the browser.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/90 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-gray-900 rounded-2xl p-8 max-w-2xl w-full border border-gray-800 shadow-2xl">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <CheckCircle className="text-green-500" size={32} />
            <h2 className="text-2xl font-bold text-white">Contract Status</h2>
          </div>
          <button
            onClick={() => setShowDetails(false)}
            className="text-gray-400 hover:text-white transition-colors"
          >
            ✕
          </button>
        </div>

        <div className="space-y-4">
          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-semibold text-gray-400">PLINK Token</span>
              <a
                href={`${explorerUrl}/${CONTRACTS.plinkToken}`}
                target="_blank"
                rel="noopener noreferrer"
                className="text-purple-400 hover:text-purple-300 transition-colors"
              >
                <ExternalLink size={16} />
              </a>
            </div>
            <code className="text-xs text-green-400 break-all">{CONTRACTS.plinkToken}</code>
          </div>

          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-semibold text-gray-400">Purchase Contract</span>
              <a
                href={`${explorerUrl}/${CONTRACTS.purchase}`}
                target="_blank"
                rel="noopener noreferrer"
                className="text-purple-400 hover:text-purple-300 transition-colors"
              >
                <ExternalLink size={16} />
              </a>
            </div>
            <code className="text-xs text-green-400 break-all">{CONTRACTS.purchase}</code>
          </div>

          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-semibold text-gray-400">Plinko Game</span>
              <a
                href={`${explorerUrl}/${CONTRACTS.game}`}
                target="_blank"
                rel="noopener noreferrer"
                className="text-purple-400 hover:text-purple-300 transition-colors"
              >
                <ExternalLink size={16} />
              </a>
            </div>
            <code className="text-xs text-green-400 break-all">{CONTRACTS.game}</code>
          </div>

          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-semibold text-gray-400">Treasury Wallet</span>
            </div>
            <code className="text-xs text-green-400 break-all">{CONTRACTS.treasury || 'Not set'}</code>
          </div>

          <div className="bg-green-900/20 border border-green-700 rounded-lg p-4">
            <p className="text-sm text-green-400">
              ✓ All contracts are deployed and active on <strong>{NETWORK}</strong>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
