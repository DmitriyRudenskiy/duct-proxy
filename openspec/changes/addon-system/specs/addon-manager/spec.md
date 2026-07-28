## ADDED Requirements

### Requirement: AddonManager registration
The system SHALL provide an `AddonManager` that manages addon registration and dispatch.

#### Scenario: Register addon
- **WHEN** `AddonManager::register(addon: Box<dyn Addon>)` is called
- **THEN** the addon is added to the internal registry

#### Scenario: Register multiple addons
- **WHEN** multiple addons are registered
- **THEN** all addons are stored and will be dispatched in registration order

### Requirement: Sequential hook dispatch
The system SHALL dispatch hooks sequentially to all registered addons.

#### Scenario: Dispatch requestheaders to all addons
- **WHEN** `dispatch_requestheaders(flow: &mut HTTPFlow)` is called
- **THEN** each addon's `requestheaders` method is called in registration order

#### Scenario: Dispatch stops on addon error
- **WHEN** an addon returns an error during dispatch
- **THEN** subsequent addons are NOT called and the error is returned

### Requirement: Error isolation
The system SHALL isolate addon errors to prevent one addon from affecting others.

#### Scenario: Addon error doesn't crash manager
- **WHEN** an addon panics or returns an error
- **THEN** the manager logs the error and continues with other addons (if configured)

#### Scenario: Error is propagated to caller
- **WHEN** an addon returns an error
- **THEN** the error is returned to the hook dispatcher in mitm-proxy

### Requirement: Hook lifecycle ordering
The system SHALL enforce the correct hook lifecycle order.

#### Scenario: Request lifecycle order
- **WHEN** an HTTP request is processed
- **THEN** hooks are called in order: requestheaders → request

#### Scenario: Response lifecycle order
- **WHEN** an HTTP response is processed
- **THEN** hooks are called in order: responseheaders → response

### Requirement: Addon manager configuration
The system SHALL support configuration options for addon behavior.

#### Scenario: Configure timeout
- **WHEN** `AddonManager::new(timeout: Duration)` is called
- **THEN** addon executions are cancelled after the timeout

#### Scenario: Configure error handling
- **WHEN** `AddonManager::set_error_policy(policy: ErrorPolicy)` is called
- **THEN** the manager uses the specified policy for error handling
