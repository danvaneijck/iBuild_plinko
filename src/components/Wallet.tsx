// src/components/Wallet.tsx

import React, { useState } from 'react';
import { Wallet as WalletIcon, X, AlertCircle } from 'lucide-react';
import { Wallet as InjectiveWallet } from '@injectivelabs/wallet-base';

// Define the props the component will accept from its parent (App.tsx)
interface WalletProps {
    address?: string;
    injectiveAddress?: string; // Prop passed from App, can be used if needed
    isConnecting: boolean;
    isConnected: boolean;
    error: string | null;
    onConnect: (wallet: InjectiveWallet) => Promise<void>;
    onDisconnect: () => void;
}

export const Wallet: React.FC<WalletProps> = ({
    address,
    isConnecting,
    isConnected,
    error,
    onConnect,
    onDisconnect,
}) => {
    const [showModal, setShowModal] = useState(false);

    const wallets = [
        { name: 'Keplr', wallet: InjectiveWallet.Keplr },
        { name: 'Leap', wallet: InjectiveWallet.Leap },
        { name: 'Metamask', wallet: InjectiveWallet.Metamask },
        { name: 'Rabby', wallet: InjectiveWallet.Rabby },
    ];

    const handleConnect = async (wallet: InjectiveWallet) => {
        await onConnect(wallet);
        // The modal will now close regardless of success,
        // allowing the parent to display any error messages.
        setShowModal(false);
    };

    const formatAddress = (addr: string | undefined) => {
        if (!addr) return '';
        return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
    };

    return (
        <>
            {!isConnected ? (
                <button
                    onClick={() => setShowModal(true)}
                    disabled={isConnecting}
                    className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-600 text-white rounded-xl font-semibold hover:from-purple-700 hover:to-pink-700 transition-all duration-200 shadow-lg hover:shadow-xl disabled:opacity-50"
                >
                    <WalletIcon size={20} />
                    {isConnecting ? 'Connecting...' : 'Connect Wallet'}
                </button>
            ) : (
                <div className="flex items-center gap-3">
                    <div className="px-4 py-2 bg-gray-800 rounded-lg border border-gray-700">
                        <span className="text-sm text-gray-400">Connected:</span>
                        <span className="ml-2 text-white font-mono">{formatAddress(address)}</span>
                    </div>
                    <button
                        onClick={onDisconnect}
                        className="p-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
                        title="Disconnect"
                    >
                        <X size={20} />
                    </button>
                </div>
            )}

            {showModal && (
                <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
                    <div className="bg-gray-900 rounded-2xl p-8 max-w-md w-full border border-gray-800 shadow-2xl animate-fade-in-up">
                        <div className="flex justify-between items-center mb-6">
                            <h2 className="text-2xl font-bold text-white">Connect Wallet</h2>
                            <button
                                onClick={() => setShowModal(false)}
                                className="p-2 hover:bg-gray-800 rounded-lg transition-colors"
                            >
                                <X size={24} className="text-gray-400" />
                            </button>
                        </div>

                        {/* Note: Errors are now displayed in the main App component, but can be shown here too if desired */}
                        {error && (
                            <div className="mb-4 p-4 bg-red-900/50 border border-red-700 rounded-lg flex items-start gap-3">
                                <AlertCircle className="text-red-500 flex-shrink-0 mt-0.5" size={20} />
                                <div className="text-sm text-red-200">{error}</div>
                            </div>
                        )}

                        <div className="space-y-3">
                            {wallets.map(({ name, wallet }) => (
                                <button
                                    key={name}
                                    onClick={() => handleConnect(wallet)}
                                    disabled={isConnecting}
                                    className="w-full p-4 bg-gray-800 hover:bg-gray-700 rounded-xl text-white font-semibold transition-all duration-200 border border-gray-700 hover:border-purple-500 disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {isConnecting ? 'Connecting...' : name}
                                </button>
                            ))}
                        </div>

                        <div className="mt-6 text-xs text-gray-500 text-center">
                            Make sure you have the wallet extension installed and are on the correct network.
                        </div>
                    </div>
                </div>
            )}
        </>
    );
};