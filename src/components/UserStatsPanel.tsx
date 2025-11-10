import { useState, useEffect } from 'react';
import { useWallet } from '../hooks/useWallet';
import { walletStrategy } from '../hooks/useContracts';
import { BigNumberInBase } from '@injectivelabs/utils';
import { User, TrendingUp, Shield, Trophy } from 'lucide-react';
import { ContractService } from '../services/contractService';

const formatStatValue = (value: string | undefined) => {
    if (!value) return '0.00';
    return new BigNumberInBase(value).toWei(-18).toFormat(2);
};

export const UserStatsPanel = () => {
    const { address, isConnected } = useWallet();
    const [stats, setStats] = useState<any>(null);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        const fetchStats = async () => {
            if (!isConnected || !address) return;
            setIsLoading(true);
            const contractService = new ContractService(walletStrategy);
            const data = await contractService.getUserStats(address);
            setStats(data);
            setIsLoading(false);
        };

        fetchStats();
        // Refresh stats every 30 seconds while connected
        const interval = setInterval(fetchStats, 30000);
        return () => clearInterval(interval);
    }, [address, isConnected]);

    const StatItem = ({ icon, label, value }: { icon: React.ReactNode, label: string, value: string }) => (
        <div className="flex justify-between items-center text-sm">
            <div className="flex items-center gap-2 text-gray-400">
                {icon}
                <span>{label}</span>
            </div>
            <span className="font-mono text-white font-semibold">{value}</span>
        </div>
    );

    if (!isConnected) {
        return (
            <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-4 text-center text-gray-500">
                Connect your wallet to see your stats.
            </div>
        )
    }

    if (isLoading && !stats) {
        return <div className="text-center p-4 text-gray-500">Loading Your Stats...</div>;
    }

    if (!stats) return

    return (
        <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-4 space-y-3 fade-in">
            <h3 className="text-base font-bold text-white mb-2">Your Stats</h3>
            <StatItem icon={<User size={16} />} label="Games Played" value={stats.total_games?.toLocaleString() || '0'} />
            <StatItem icon={<TrendingUp size={16} />} label="Total Wagered" value={`${formatStatValue(stats.total_wagered)} PLINK`} />
            <StatItem icon={<Shield size={16} />} label="Total Won" value={`${formatStatValue(stats.total_won)} PLINK`} />
            <StatItem icon={<Trophy size={16} />} label="Best Win (PNL)" value={`${formatStatValue(stats.best_win_pnl)} PLINK`} />
            <StatItem icon={<Trophy size={16} />} label="Best Multiplier" value={stats.best_win_multiplier || 'N/A'} />
        </div>
    );
};