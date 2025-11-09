import { useState, useEffect, useCallback } from 'react';
import { useWallet } from '../hooks/useWallet'; // To know if the wallet is connected
import { walletStrategy } from '../hooks/useContracts'; // Import the shared walletStrategy instance
import { BigNumberInBase } from '@injectivelabs/utils';
import { Crown, Shield, TrendingUp } from 'lucide-react';
import { ContractService } from '../services/contractService';

// Define types for clarity
type LeaderboardScope = 'global' | 'daily';
type LeaderboardType = 'bestWins' | 'totalWagered';

interface LeaderboardEntry {
    player: string;
    value: string; // This is a Uint128 string
    multiplier?: string;
}

// Helper function to truncate wallet addresses
const truncateAddress = (address: string) => `${address.slice(0, 6)}...${address.slice(-4)}`;

// Helper to format the large number strings from the contract
const formatValue = (value: string, type: LeaderboardType) => {
    const formatted = new BigNumberInBase(value).toWei(-18).toFormat(2);
    return `${formatted} ${type === 'bestWins' ? 'PNL' : ''}`;
};

export const Leaderboard = () => {
    const { isConnected } = useWallet();
    const [scope, setScope] = useState<LeaderboardScope>('global');
    const [type, setType] = useState<LeaderboardType>('bestWins');
    const [entries, setEntries] = useState<LeaderboardEntry[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchLeaderboard = useCallback(async () => {
        if (!isConnected) return;

        setIsLoading(true);
        setError(null);
        setEntries([]);

        try {
            const contractService = new ContractService(walletStrategy);
            let data;
            if (scope === 'global') {
                data = await contractService.getGlobalLeaderboard(type, 10);
            } else {
                data = await contractService.getDailyLeaderboard(type, 10);
            }
            setEntries(data);
        } catch (err) {
            console.error("Failed to fetch leaderboard", err);
            setError("Could not load leaderboard data.");
        } finally {
            setIsLoading(false);
        }
    }, [isConnected, scope, type]);

    useEffect(() => {
        fetchLeaderboard();
    }, [fetchLeaderboard]);

    const renderEntries = () => {
        if (isLoading) {
            return <div className="text-center p-8 text-gray-400">Loading...</div>;
        }
        if (error) {
            return <div className="text-center p-8 text-red-400">{error}</div>;
        }
        if (entries.length === 0) {
            return <div className="text-center p-8 text-gray-400">No entries yet. Be the first!</div>;
        }

        return (
            <ul className="space-y-2">
                {entries.map((entry, index) => (
                    <li key={index} className="flex items-center justify-between bg-gray-900/50 p-3 rounded-lg">
                        <div className="flex items-center gap-4">
                            <span className={`font-bold w-6 text-center ${index < 3 ? 'text-purple-400' : 'text-gray-500'}`}>
                                {index + 1}
                            </span>
                            <span className="font-mono text-sm text-gray-300">{truncateAddress(entry.player)}</span>
                        </div>
                        <div className="text-right">
                            <p className="font-semibold text-white">{formatValue(entry.value, type)}</p>
                            {entry.multiplier && <p className="text-xs text-pink-400">{entry.multiplier}</p>}
                        </div>
                    </li>
                ))}
            </ul>
        );
    };

    return (
        <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-4 space-y-4">
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
                <Crown size={20} className="text-yellow-400" />
                Leaderboard
            </h3>

            {/* Scope Tabs: Global / Daily */}
            <div className="flex bg-gray-900 p-1 rounded-md">
                <button
                    onClick={() => setScope('global')}
                    className={`flex-1 p-2 text-sm rounded ${scope === 'global' ? 'bg-purple-600 text-white font-semibold' : 'text-gray-400 hover:bg-gray-700'}`}
                >
                    Global
                </button>
                <button
                    onClick={() => setScope('daily')}
                    className={`flex-1 p-2 text-sm rounded ${scope === 'daily' ? 'bg-purple-600 text-white font-semibold' : 'text-gray-400 hover:bg-gray-700'}`}
                >
                    Daily
                </button>
            </div>

            {/* Type Tabs: Best Wins / Total Wagered */}
            <div className="flex border border-gray-700 p-1 rounded-md">
                <button
                    onClick={() => setType('bestWins')}
                    className={`flex-1 p-2 text-xs rounded flex items-center justify-center gap-2 ${type === 'bestWins' ? 'bg-gray-700 text-white font-semibold' : 'text-gray-400 hover:bg-gray-600'}`}
                >
                    <TrendingUp size={14} />
                    Best Wins
                </button>
                <button
                    onClick={() => setType('totalWagered')}
                    className={`flex-1 p-2 text-xs rounded flex items-center justify-center gap-2 ${type === 'totalWagered' ? 'bg-gray-700 text-white font-semibold' : 'text-gray-400 hover:bg-gray-600'}`}
                >
                    <Shield size={14} />
                    Total Wagered
                </button>
            </div>

            {/* Entries List */}
            <div className="h-80 overflow-y-auto pr-2">
                {renderEntries()}
            </div>
        </div>
    );
};