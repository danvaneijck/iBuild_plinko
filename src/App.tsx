import { useState } from 'react';
import { Wallet } from './components/Wallet';
import { PlinkoBoard } from './components/PlinkoBoard';
import { GameControls } from './components/GameControls';
import { GameHistory } from './components/GameHistory';
import { PurchaseModal } from './components/PurchaseModal';
import { ContractStatus } from './components/ContractStatus';
import { useWallet } from './hooks/useWallet';
import { usePlinkoGame } from './hooks/usePlinkoGame';
import { Coins } from 'lucide-react';

function App() {
  const { address, injectiveAddress, isConnecting, selectedWallet, error: walletError, connect, disconnect, isConnected } = useWallet();
  const { 
    balls, 
    gameHistory, 
    plinkBalance, 
    isLoading,
    error: gameError,
    contractsValid,
    dropBall, 
    purchasePlink,
    refreshBalance 
  } = usePlinkoGame(address);
  
  const [isPurchaseModalOpen, setIsPurchaseModalOpen] = useState(false);

  const handlePurchase = async (injAmount: string) => {
    try {
      await purchasePlink(injAmount);
      await refreshBalance();
    } catch (err) {
      console.error('Purchase failed:', err);
      throw err;
    }
  };

  const handlePlay = async (difficulty: any, riskLevel: any, betAmount: string) => {
    try {
      await dropBall(difficulty, riskLevel, betAmount);
    } catch (err) {
      console.error('Game failed:', err);
      throw err;
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-purple-900 to-gray-900">
      <ContractStatus />
      
      {/* Header */}
      <header className="border-b border-gray-800 bg-gray-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 bg-gradient-to-br from-purple-600 to-pink-600 rounded-xl flex items-center justify-center shadow-lg">
                <Coins className="text-white" size={28} />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-white">$PLINKO</h1>
                <p className="text-sm text-gray-400">Blockchain Plinko Game</p>
              </div>
            </div>

            <div className="flex items-center gap-4">
              {isConnected && (
                <button
                  onClick={() => setIsPurchaseModalOpen(true)}
                  disabled={!contractsValid || isLoading}
                  className="px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white font-bold rounded-xl transition-all duration-200 shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Buy $PLINK
                </button>
              )}
              <Wallet
                address={address}
                injectiveAddress={injectiveAddress}
                isConnecting={isConnecting}
                selectedWallet={selectedWallet}
                error={walletError}
                onConnect={connect}
                onDisconnect={disconnect}
              />
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8">
        {!contractsValid && (
          <div className="mb-6 bg-yellow-900/20 border border-yellow-700 rounded-xl p-4">
            <p className="text-yellow-400 text-sm">
              ⚠️ Smart contracts not configured. Please deploy contracts and update .env file.
            </p>
          </div>
        )}

        {walletError && (
          <div className="mb-6 bg-red-900/20 border border-red-700 rounded-xl p-4">
            <p className="text-red-400 text-sm">{walletError}</p>
          </div>
        )}

        {gameError && (
          <div className="mb-6 bg-red-900/20 border border-red-700 rounded-xl p-4">
            <p className="text-red-400 text-sm">{gameError}</p>
          </div>
        )}

        {!isConnected ? (
          <div className="text-center py-20">
            <div className="w-24 h-24 bg-gradient-to-br from-purple-600 to-pink-600 rounded-full flex items-center justify-center mx-auto mb-6 shadow-2xl">
              <Coins className="text-white" size={48} />
            </div>
            <h2 className="text-3xl font-bold text-white mb-4">Welcome to $PLINKO</h2>
            <p className="text-gray-400 mb-8 max-w-md mx-auto">
              Connect your wallet to start playing the blockchain Plinko game with provably fair results
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Left Column - Game Controls */}
            <div className="space-y-6">
              <GameControls
                onPlay={handlePlay}
                plinkBalance={plinkBalance}
                disabled={!contractsValid}
                isLoading={isLoading}
              />
              <GameHistory games={gameHistory} />
            </div>

            {/* Center Column - Plinko Board */}
            <div className="lg:col-span-2">
              <PlinkoBoard balls={balls} />
            </div>
          </div>
        )}
      </main>

      {/* Purchase Modal */}
      <PurchaseModal
        isOpen={isPurchaseModalOpen}
        onClose={() => setIsPurchaseModalOpen(false)}
        onPurchase={handlePurchase}
        isLoading={isLoading}
      />
    </div>
  );
}

export default App;
