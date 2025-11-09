import { useState } from 'react';
import { Wallet } from './components/Wallet';
import { PlinkoBoard } from './components/PlinkoBoard';
import { GameControls } from './components/GameControls';
import { GameHistory } from './components/GameHistory';
import { PurchaseModal } from './components/PurchaseModal';
import { useWallet } from './hooks/useWallet';
import { usePlinkoGame } from './hooks/usePlinkoGame';
import { Coins } from 'lucide-react';
import { Difficulty, RiskLevel } from './types/game';
import { MULTIPLIERS } from './config/multipliers';
import { Leaderboard } from './components/Leaderboard';
import { UserStatsPanel } from './components/UserStatsPanel';
import { GameStatsPanel } from './components/GameStatsPanel';

function App() {
  const { address, injectiveAddress, isConnecting, error: walletError, connect, disconnect, isConnected } = useWallet();
  const {
    balls,
    gameHistory,
    plinkBalance,
    isLoading,
    error: gameError,
    contractsValid,
    dropBall,
    purchasePlink,
    refreshBalance,
    onAnimationComplete
  } = usePlinkoGame(address);


  // --- Add state for game settings ---
  const [difficulty, setDifficulty] = useState<Difficulty>('medium');
  const [riskLevel, setRiskLevel] = useState<RiskLevel>('medium');
  const [isPurchaseModalOpen, setIsPurchaseModalOpen] = useState(false);

  // --- Derive multipliers from state ---
  const multipliers = MULTIPLIERS[difficulty][riskLevel];

  const handlePurchase = async (injAmount: string) => {
    try {
      await purchasePlink(injAmount);
      await refreshBalance();
    } catch (err) {
      console.error('Purchase failed:', err);
      throw err;
    }
  };

  // The handlePlay function now uses the state variables
  const handlePlay = async (betAmount: string, numberOfBalls: number) => {
    try {
      // Pass all arguments to the dropBall hook function
      await dropBall(difficulty, riskLevel, betAmount, numberOfBalls);
    } catch (err) {
      console.error('Game failed:', err);
      // Optionally, show this error in the UI
    }
  };

  const renderErrorMessages = () => (
    <>
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
    </>
  );

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-purple-900 to-gray-900 text-white">
      {/* Header */}
      <header className="border-b border-gray-800 bg-gray-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-screen-2xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex flex-col md:flex-row items-center justify-between">
            <div className="flex items-center gap-3 mb-2 md:mb-0">
              <div className="w-12 h-12 bg-gradient-to-br from-purple-600 to-pink-600 rounded-xl flex items-center justify-center shadow-lg">
                <Coins size={28} />
              </div>
              <div>
                <h1 className="text-2xl font-bold">$PLINK</h1>
                <p className="text-sm text-gray-400">Provably Fair Plinko on Injective</p>
              </div>
            </div>
            <div className="flex items-center gap-4">
              {isConnected && (
                <button
                  onClick={() => setIsPurchaseModalOpen(true)}
                  disabled={!contractsValid || isLoading}
                  className=" text-xs md:text-sm px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 font-bold rounded-xl transition-all duration-200 shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Buy $PLINK
                </button>
              )}
              <Wallet
                address={address}
                injectiveAddress={injectiveAddress}
                isConnecting={isConnecting}
                isConnected={isConnected}
                error={walletError}
                onConnect={connect}
                onDisconnect={disconnect}
              />
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-screen-2xl mx-auto px-4 sm:px-6 lg:px-8 py-8"> {/* Increased max-width */}
        {renderErrorMessages()}

        {!isConnected ? (
          <div className="text-center py-20">
            {/* ... same welcome message for logged out users ... */}
          </div>
        ) : (

          <div className="flex flex-col lg:grid lg:grid-cols-12 lg:gap-8">

            {/* --- Mobile & Desktop Left Column: Game Controls --- */}
            {/* 'order-1' makes it appear first on mobile */}
            <div className="lg:col-span-3 order-1 lg:order-1 mb-8 lg:mb-0">
              <GameControls
                onPlay={handlePlay}
                plinkBalance={plinkBalance}
                disabled={!contractsValid}
                isLoading={isLoading}
                difficulty={difficulty}
                riskLevel={riskLevel}
                onDifficultyChange={setDifficulty}
                onRiskLevelChange={setRiskLevel}
              />
            </div>

            {/* --- Mobile & Desktop Center Column: Plinko Board --- */}
            {/* 'order-2' makes it appear second on mobile */}
            <div className="lg:col-span-6 order-2 lg:order-2 mb-8 lg:mb-0">
              <PlinkoBoard
                balls={balls}
                difficulty={difficulty}
                multipliers={multipliers}
                onAnimationComplete={onAnimationComplete}
              />
            </div>

            {/* --- Mobile & Desktop Right Column: Stats & Info --- */}
            {/* 'order-3' makes it appear last on mobile */}
            <div className="lg:col-span-3 order-3 lg:order-3 space-y-6">
              <UserStatsPanel />
              <GameStatsPanel />
              <Leaderboard />
            </div>

            {/* --- Game History (Full Width Below) --- */}
            {/* This will span the full width of the grid on desktop */}
            <div className="lg:col-span-12 order-4 mt-8">
              {gameHistory && gameHistory.length > 0 &&
                <GameHistory history={gameHistory} />
              }
            </div>
          </div>
        )}
      </main>

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
