import { useState, useEffect } from "react";

/**
 * A custom React hook that tracks the browser window's width.
 * It gracefully handles Server-Side Rendering (SSR) by only
 * accessing the `window` object on the client side.
 *
 * @returns {number} The current width of the window in pixels.
 */
export const useWindowWidth = (): number => {
    // Initialize state with 0, or a default width.
    // We don't use window.innerWidth here because it would cause a crash on the server
    // where the `window` object does not exist.
    const [width, setWidth] = useState<number>(0);

    useEffect(() => {
        // This handler function will be called whenever the window is resized.
        const handleResize = () => {
            setWidth(window.innerWidth);
        };

        // --- Critical Part ---
        // This effect only runs on the client side, after the component has mounted.
        // This is where we can safely access `window`.

        // 1. Set the initial width
        handleResize();

        // 2. Add the event listener for future resize events
        window.addEventListener("resize", handleResize);

        // 3. Return a cleanup function
        // This function will be called when the component unmounts.
        // It's crucial for preventing memory leaks by removing the event listener.
        return () => {
            window.removeEventListener("resize", handleResize);
        };
    }, []); // The empty dependency array `[]` ensures this effect runs only once on mount.

    return width;
};
