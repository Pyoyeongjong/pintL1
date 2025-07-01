//! Configs for Transaction Pool
pub struct PoolConfig {
    pub max_new_pending_txs_notifications: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_new_pending_txs_notifications: 1,
        }
    }
}
