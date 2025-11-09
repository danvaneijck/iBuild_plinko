import { useState, useEffect } from 'react';
import { walletStrategy } from '../hooks/useContracts';
import { BigNumberInBase } from '@injectivelabs/utils';
import { TrendingUp, Shield, ToyBrick, Scale } from 'lucide-react';
import { ContractService } from '../services/contractService';

// A helper to format the large Uint128 values nicely
const formatStatValue = (value: string | undefined) => {
    if (!value) return '0.00';
    return new BigNumberInBase(value).toWei(-18).toFormat(2);
};

export const GameStatsPanel = () => {
    const [stats, setStats] = useState<any>({});
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        const fetchStats = async () => {
            setIsLoading(true);
            const contractService = new ContractService(walletStrategy);
            const data = await contractService.getGameStats();
            setStats(data);
            setIsLoading(false);
        };

        fetchStats();
        // Optional: Refresh stats every 30 seconds
        const interval = setInterval(fetchStats, 30000);
        return () => clearInterval(interval);
    }, []);

    const StatItem = ({ icon, label, value }: { icon: React.ReactNode, label: string, value: string }) => (
        <div className="flex justify-between items-center text-sm">
            <div className="flex items-center gap-2 text-gray-400">
                {icon}
                <span>{label}</span>
            </div>
            <span className="font-mono text-white font-semibold">{value}</span>
        </div>
    );

    if (isLoading && !stats) {
        return <div className="text-center p-4 text-gray-500">Loading Game Stats...</div>;
    }

    if (!stats) return

    return (
        <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-4 space-y-3">
            <h3 className="text-base font-bold text-white mb-2">Game Stats</h3>
            <StatItem icon={<ToyBrick size={16} />} label="Total Games Played" value={stats.total_games?.toLocaleString() || '0'} />
            <StatItem icon={<TrendingUp size={16} />} label="Total Wagered" value={`${formatStatValue(stats.total_wagered)} PLINK`} />
            <StatItem icon={<Shield size={16} />} label="Total Won" value={`${formatStatValue(stats.total_won)} PLINK`} />
            <StatItem icon={<Scale size={16} />} label="House Balance" value={`${formatStatValue(stats.house_balance)} PLINK`} />
        </div>
    );
};