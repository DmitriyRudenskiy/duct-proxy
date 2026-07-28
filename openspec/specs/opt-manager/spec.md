# OptManager Specification

## Purpose

Define the `OptManager` for thread-safe runtime options management with validation and change notifications.

## Requirements

### Requirement: OptManager struct

The system SHALL provide an `OptManager` struct that wraps `Arc<RwLock<Options>>` for thread-safe runtime options management.

#### Scenario: OptManager can be created from Options
- **WHEN** user creates `OptManager::new(options)`
- **THEN** the manager stores a clone of options in `Arc<RwLock<Options>>`

#### Scenario: OptManager can be cloned
- **WHEN** user clones an OptManager
- **THEN** the clone shares the same underlying `Arc<RwLock<Options>>`

### Requirement: Get options

The system SHALL provide a `get()` method to retrieve a snapshot of current options.

#### Scenario: Get returns current options
- **WHEN** user calls `manager.get()`
- **THEN** the system returns a clone of the current `Options`

#### Scenario: Get is thread-safe
- **WHEN** multiple threads call `get()` concurrently
- **THEN** all threads receive consistent snapshots

### Requirement: Set options

The system SHALL provide a `set()` method to update options at runtime.

#### Scenario: Set updates options
- **WHEN** user calls `manager.set(new_options)`
- **THEN** the manager updates the stored options

#### Scenario: Set validates options
- **WHEN** user calls `manager.set(invalid_options)`
- **THEN** the system returns an `Err` if validation fails

#### Scenario: Set is thread-safe
- **WHEN** multiple threads call `set()` concurrently
- **THEN** updates are serialized via `RwLock`

### Requirement: Option validation

The system SHALL validate options before accepting them.

#### Scenario: Port validation
- **WHEN** user sets `listen_port` to `0` or `> 65535`
- **THEN** the system returns a `ValidationError`

#### Scenario: Host validation
- **WHEN** user sets `listen_host` to an invalid IP
- **THEN** the system returns a `ValidationError`

#### Scenario: Mode validation
- **WHEN** user sets `mode` to an invalid value
- **THEN** the system returns a `ValidationError` (should not happen with enum)

### Requirement: Change notifications (future)

The system SHALL support optional change notifications when options are updated.

#### Scenario: Change notification callback
- **WHEN** user registers a callback via `on_change(fn)`
- **THEN** the callback is invoked after `set()` succeeds

#### Scenario: No notifications by default
- **WHEN** user does not register a callback
- **THEN** `set()` works without notification overhead
