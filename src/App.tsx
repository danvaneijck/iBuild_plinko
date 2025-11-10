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
import { NETWORK } from './config/contracts';
import NetworkDisplay from './components/NetworkDisplay';

function App() {
  const { address, injectiveAddress, isConnecting, error: walletError, connect, disconnect, isConnected } = useWallet();

  // --- Add state for game settings ---
  const [difficulty, setDifficulty] = useState<Difficulty>('medium');
  const [riskLevel, setRiskLevel] = useState<RiskLevel>('medium');
  const [isPurchaseModalOpen, setIsPurchaseModalOpen] = useState(false);

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
  } = usePlinkoGame(address, difficulty, riskLevel);

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
              <NetworkDisplay network={NETWORK} />

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
          <div className="text-center py-2 md:py-10">
            <div className="w-24 h-24 bg-gradient-to-br from-purple-600 to-pink-600 rounded-full flex items-center justify-center mx-auto mb-6 shadow-2xl">
              <Coins className="text-white" size={48} />
            </div>
            <h2 className="text-3xl font-bold text-white mb-4">Welcome to $PLINKO</h2>
            <p className="text-gray-400 mb-8 max-w-md mx-auto">
              Connect your wallet to start playing the blockchain Plinko game with provably fair results
            </p>
            <div className='max-w-screen-lg m-auto space-y-4'>
              <GameStatsPanel />
              <Leaderboard />
            </div>

          </div>
        ) : (

          <div className="flex flex-col lg:grid lg:grid-cols-12 lg:gap-8">

            {/* --- Mobile & Desktop Left Column: Game Controls --- */}
            {/* 'order-1' makes it appear first on mobile */}
            <div className="lg:col-span-3 order-1 lg:order-1 mb-8 lg:mb-0 space-y-6">
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
              {/* --- Game History for Desktop --- */}
              {gameHistory && gameHistory.length > 0 &&
                <div className="hidden lg:block">
                  <GameHistory history={gameHistory} />
                </div>
              }
            </div>

            {/* --- Mobile & Desktop Center Column: Plinko Board --- */}
            {/* 'order-2' makes it appear second on mobile */}
            <div className="lg:col-span-6 order-2 lg:order-2 mb-8 lg:mb-0">
              <PlinkoBoard
                balls={balls}
                difficulty={difficulty}
                multipliers={multipliers}
                gameHistory={gameHistory}
                onAnimationComplete={onAnimationComplete}
              />
              <div className='flex justify-center mt-4 w-full gap-2 fade-in'>
                <a href="https://ibuild.dev/" target="_blank" rel="noopener noreferrer" className="ibuild-badge-dark">
                  <span>Built using</span>
                  <img src="https://ibuild.dev/icons/ibuild_logo.svg" alt="iBuild Logo" />
                </a>
                <a href="https://injective.com/" target="_blank" rel="noopener noreferrer" className="inj-badge-dark">
                  <span>Powered by Injective</span>
                  <svg className="inj-logo" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M14.5395 0.276367L14.4675 0.420367C16.9635 1.50037 18.3795 3.73237 18.3795 6.18037C18.3795 8.82037 16.6515 11.2204 13.2915 13.2604L12.7395 13.5964C10.2195 15.1324 8.94753 16.8604 8.94753 18.9484C8.94753 21.6364 11.0595 23.6043 13.9635 23.6043C18.6195 23.6043 24.0195 18.6364 24.0195 12.0124C24.0195 10.9324 23.8755 9.87637 23.6115 8.86837L23.4435 8.91637C23.5155 9.46837 23.5395 9.87637 23.5395 10.2604C23.5395 14.1724 21.3315 17.5324 17.6355 19.7644L17.2755 19.9804C16.4835 20.4364 15.8115 20.7004 15.1395 20.7004C14.2515 20.7004 13.5795 20.1244 13.5795 19.2844C13.5795 18.5644 14.0595 17.8444 15.6195 16.9324L16.0755 16.6684C19.5315 14.6524 21.4515 11.8204 21.4515 8.67637C21.4515 4.69237 18.4755 1.40437 14.5395 0.276367ZM9.49953 23.7243L9.57153 23.5803C7.07553 22.5004 5.65953 20.2684 5.65953 17.8204C5.65953 15.1804 7.38753 12.7804 10.7475 10.7404L11.2995 10.4044C13.8195 8.86837 15.0915 7.14037 15.0915 5.05237C15.0915 2.36437 12.9795 0.396367 10.0755 0.396367C5.41953 0.396367 0.0195312 5.36437 0.0195312 11.9884C0.0195312 13.0684 0.163532 14.1244 0.427532 15.1324L0.595532 15.0844C0.523532 14.5324 0.499531 14.1244 0.499531 13.7404C0.499531 9.82837 2.70753 6.46837 6.40353 4.23637L6.76353 4.02037C7.55553 3.56437 8.22753 3.30037 8.89953 3.30037C9.78753 3.30037 10.4595 3.87637 10.4595 4.71637C10.4595 5.43637 9.97953 6.15637 8.41953 7.06837L7.96353 7.33237C4.50753 9.34837 2.58753 12.1804 2.58753 15.3244C2.58753 19.3084 5.56353 22.5964 9.49953 23.7243Z" fill="currentColor"></path>
                  </svg>
                </a>
              </div>
            </div>

            {/* --- Mobile & Desktop Right Column: Stats & Info --- */}
            {/* 'order-3' makes it appear last on mobile */}
            <div className="lg:col-span-3 order-3 lg:order-3 space-y-6">
              <UserStatsPanel />
              <GameStatsPanel />
              {/* --- Game History for Mobile --- */}
              {gameHistory && gameHistory.length > 0 &&
                <div className="lg:hidden">
                  <GameHistory history={gameHistory} />
                </div>
              }
              <Leaderboard />
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
