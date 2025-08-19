use clap::ValueEnum;

/// Enum for specifying how to handle transactions output.
/// - `SendActual` sends the actual transaction to the cluster
/// - `SimOnly` simulates the transaction against the cluster
/// - `Dump64` outputs base64 encoded serialized transaction to stdout for use with multisigs, explorer inspectors, or piping into other applications
/// - `Dump58` outputs base58 encoded serialized transaction to stdout for use with multisigs, explorer inspectors, or piping into other applications
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum TxSendMode {
    #[default]
    SendActual,
    SimOnly,
    Dump64,
    Dump58,
}
