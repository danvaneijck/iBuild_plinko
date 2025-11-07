/**
 * Converts a raw token amount (as a string) to a human-readable format.
 * @param amount The raw amount from the contract (e.g., "1000000000000000000").
 * @param decimals The number of decimals the token uses (e.g., 18).
 * @param precision The number of decimal places to show in the output.
 * @returns A formatted string (e.g., "1.00").
 */
export const formatTokenAmount = (
    amount: string | undefined,
    decimals: number,
    precision: number = 2
): string => {
    if (!amount || isNaN(parseInt(amount))) {
        return "0.00";
    }

    try {
        // Use BigInt for precise arithmetic with large numbers
        const amountBigInt = BigInt(amount);
        const divisor = BigInt(10) ** BigInt(decimals);

        const wholePart = amountBigInt / divisor;
        const fractionalPart = amountBigInt % divisor;

        // Pad the fractional part with leading zeros to ensure it has `decimals` length
        const fractionalString = fractionalPart
            .toString()
            .padStart(decimals, "0");

        // Combine and format to the desired precision
        const fullNumberString = `${wholePart}.${fractionalString}`;

        return parseFloat(fullNumberString).toFixed(precision);
    } catch (error) {
        console.error("Failed to format token amount:", error);
        return "0.00";
    }
};
