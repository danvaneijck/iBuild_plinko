import { useState, useCallback, useMemo, useEffect } from "react";
import { Wallet } from "@injectivelabs/wallet-base";
import { getInjectiveAddress } from "@injectivelabs/sdk-ts";
import { walletStrategy } from "./useContracts";

export const useWallet = () => {
    const [address, setAddress] = useState<string>("");
    const [isConnecting, setIsConnecting] = useState(false);
    const [selectedWallet, setSelectedWallet] = useState<Wallet | null>(null);
    const [error, setError] = useState<string>("");

    const injectiveAddress = useMemo(() => {
        if (address) {
            try {
                return getInjectiveAddress(address);
            } catch (e) {
                console.error("Error converting address:", e);
                return address; // Return original if conversion fails
            }
        }
        return "";
    }, [address]);

    const connect = useCallback(async (wallet: Wallet) => {
        console.log("Connecting to wallet:", wallet);
        setIsConnecting(true);
        setError("");

        try {
            // Set the wallet first
            walletStrategy.setWallet(wallet);
            console.log("Wallet set successfully");

            // Get addresses
            const addresses = await walletStrategy.getAddresses();
            console.log("Addresses received:", addresses);

            if (addresses && addresses.length > 0) {
                const walletAddress = addresses[0];
                console.log("Setting address:", walletAddress);

                // Force state update
                setAddress(walletAddress);
                setSelectedWallet(wallet);

                // Store in localStorage for persistence
                localStorage.setItem("connectedWallet", wallet);
                localStorage.setItem("walletAddress", walletAddress);

                console.log("Wallet connected successfully!");
            } else {
                throw new Error("No addresses returned from wallet");
            }
        } catch (err: any) {
            console.error("Failed to connect wallet:", err);
            setError(err.message || "Failed to connect wallet");
            setAddress("");
            setSelectedWallet(null);
        } finally {
            setIsConnecting(false);
        }
    }, []);

    const disconnect = useCallback(() => {
        console.log("Disconnecting wallet");
        setAddress("");
        setSelectedWallet(null);
        setError("");
        localStorage.removeItem("connectedWallet");
        localStorage.removeItem("walletAddress");
    }, []);

    // Auto-reconnect on mount if previously connected
    useEffect(() => {
        const savedWallet = localStorage.getItem(
            "connectedWallet"
        ) as Wallet | null;
        const savedAddress = localStorage.getItem("walletAddress");

        if (savedWallet && savedAddress) {
            console.log("Auto-reconnecting to saved wallet");
            setAddress(savedAddress);
            setSelectedWallet(savedWallet);
            walletStrategy.setWallet(savedWallet);
        }
    }, []);

    return {
        address,
        injectiveAddress,
        isConnecting,
        selectedWallet,
        error,
        connect,
        disconnect,
        isConnected: !!address,
    };
};
