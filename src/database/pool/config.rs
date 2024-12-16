#[derive(Debug, serde::Deserialize, Clone)]
pub struct DbConfig {
    /// Database-specific connection and configuration URL.
    ///
    /// The format of the URL is database specific; consult your database's
    /// documentation.
    pub url: String,
    /// Minimum number of connections to maintain in the pool.
    ///
    /// **Note:** `deadpool` drivers do not support and thus ignore this value.
    ///
    /// _Default:_ `1`.
    pub min_connections: u32,
    /// Maximum number of connections to maintain in the pool.
    ///
    /// _Default:_ `workers * 4`.
    pub max_connections: usize,
    /// Number of seconds to wait for a connection before timing out.
    ///
    /// If the timeout elapses before a connection can be made or retrieved from
    /// a pool, an error is returned.
    ///
    /// _Default:_ `5`.
    pub connect_timeout: u64,
    /// Maximum number of seconds to keep a connection alive for.
    ///
    /// After a connection is established, it is maintained in a pool for
    /// efficient connection retrieval. When an `idle_timeout` is set, that
    /// connection will be closed after the timeout elapses. If an
    /// `idle_timeout` is not specified, the behavior is driver specific but
    /// typically defaults to keeping a connection active indefinitely.
    ///
    /// _Default:_ `60`.
    pub idle_timeout: u64,
    /// The type of credentials to use when connecting to the database.
    ///
    /// _Default:_ `None`.
    pub credentials_type: String,
    /// The username to use when connecting to the database.
    ///
    /// _Default:_ `None`.
    pub user: Option<String>,
    /// The password to use when connecting to the database.
    ///
    /// _Default:_ `None`.
    pub pass: Option<String>,
    /// The namespace to use when connecting to the database.
    ///
    /// _Default:_ `None`.
    pub ns: String,
    /// The database to use when connecting to the database.
    ///
    /// _Default:_ `None`.
    pub db: String,
    /// The parameters to use when connecting to the database.
    ///
    /// _Default:_ `None`.
    pub params: Option<serde_json::Value>,
}
