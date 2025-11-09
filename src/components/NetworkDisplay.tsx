
// Define network configurations
const networkConfig = {
    mainnet: {
        className: 'network-mainnet',
        label: 'Mainnet',
    },
    testnet: {
        className: 'network-testnet',
        label: 'Testnet',
    },
    unsupported: {
        className: 'network-unsupported',
        label: 'Unsupported',
    }
};

const NetworkDisplay = ({ network = 'unsupported' }) => {
    // Get the current network's config, defaulting to unsupported
    const currentNetwork = networkConfig[network] || networkConfig.unsupported;

    return (
        <div className={`network-display ${currentNetwork.className}`}>
            <span className="network-status-dot"></span>
            <span className="network-name">{currentNetwork.label}</span>
        </div>
    );
};

export default NetworkDisplay;