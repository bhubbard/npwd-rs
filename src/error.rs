use thiserror::Error;

/// Result type alias for `npwd-rs` operations.
pub type Result<T> = std::result::Result<T, NpwdError>;

/// Core error types for the simulated phone operating system and applications.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NpwdError {
    #[error("Device is powered off")]
    DevicePoweredOff,

    #[error("Device is locked")]
    DeviceLocked,

    #[error("Battery is depleted")]
    BatteryDepleted,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Application '{0}' not found in registry")]
    AppNotFound(String),

    #[error("Application '{0}' already exists in registry")]
    AppAlreadyExists(String),

    #[error("Invalid state transition from '{from}' to '{to}': {reason}")]
    InvalidStateTransition {
        from: String,
        to: String,
        reason: String,
    },

    #[error("Permission '{0}' denied for application")]
    PermissionDenied(String),

    #[error("Contact not found: {0}")]
    ContactNotFound(String),

    #[error("Conversation not found: {0}")]
    ConversationNotFound(String),

    #[error("Insufficient funds: current balance {balance} cents, required {required} cents")]
    InsufficientFunds { balance: i64, required: i64 },

    #[error("Bank account not found: {0}")]
    AccountNotFound(String),

    #[error("Marketplace listing not found: {0}")]
    ListingNotFound(String),

    #[error("Serialization/Deserialization error: {0}")]
    SerializationError(String),

    #[error("Internal OS error: {0}")]
    Internal(String),
}
