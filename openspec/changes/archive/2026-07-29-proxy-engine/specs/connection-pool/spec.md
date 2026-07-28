## ADDED Requirements

### Requirement: Pool upstream TCP connections
The connection pool SHALL cache established TCP connections to upstream servers for reuse.

#### Scenario: Connection is reused for subsequent requests
- **WHEN** first request to example.com completes
- **THEN** the TCP connection is stored in the pool

#### Scenario: Second request reuses cached connection
- **WHEN** second request to example.com arrives within TTL
- **THEN** proxy uses cached connection instead of establishing new one

### Requirement: Evict least recently used connections
The connection pool SHALL evict connections that haven't been used recently when capacity is reached.

#### Scenario: Evict oldest connection when full
- **WHEN** pool has 100 entries and new connection is needed
- **THEN** least recently used connection is removed

#### Scenario: Recently used connections stay
- **WHEN** connection was used 1 second ago
- **THEN** connection remains in pool

### Requirement: Limit pool size per target
The connection pool SHALL enforce a maximum of 100 connections per (host, port) pair.

#### Scenario: Reject new connection when limit reached
- **WHEN** 100 connections to example.com:443 exist
- **THEN** new request creates new connection and evicts LRU

#### Scenario: Different targets have separate limits
- **WHEN** 100 connections to example.com and 50 to api.com
- **THEN** both targets can have different pool sizes

### Requirement: Close idle connections after TTL
The connection pool SHALL close connections that haven't been used within a configurable TTL.

#### Scenario: Close connection after 60 seconds idle
- **WHEN** connection hasn't been used for 60 seconds
- **THEN** connection is closed and removed from pool

#### Scenario: Active connections stay open
- **WHEN** connection is currently being used
- **THEN** connection remains in pool
