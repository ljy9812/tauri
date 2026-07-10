## ADDED Requirements

### Requirement: Auto test — register and isRegistered
The test suite SHALL include an auto test that calls `register('CommandOrControl+Shift+T', handler)`, then `isRegistered('CommandOrControl+Shift+T')`, and asserts the result is `true`. The handler callback SHALL be a no-op function.

#### Scenario: Successful registration check
- **WHEN** the test calls `register('CommandOrControl+Shift+T', () => {})` followed by `isRegistered('CommandOrControl+Shift+T')`
- **THEN** `isRegistered` returns `true`

### Requirement: Auto test — unregister and isRegistered
The test suite SHALL include an auto test that calls `register(...)`, then `unregister(...)`, then `isRegistered(...)`, and asserts the result is `false`.

#### Scenario: Successful unregistration check
- **WHEN** the test registers, then unregisters, then checks `isRegistered`
- **THEN** `isRegistered` returns `false`

### Requirement: Auto test — unregisterAll
The test suite SHALL include an auto test that calls `register(...)`, then `unregisterAll()`, then `isRegistered(...)`, and asserts the result is `false`.

#### Scenario: Successful unregisterAll check
- **WHEN** the test registers a shortcut, calls `unregisterAll()`, then checks `isRegistered`
- **THEN** `isRegistered` returns `false`

### Requirement: Side-effect test — multiple register/unregister cycles
The test suite SHALL include a side-effect test that performs multiple register/unregister cycles and asserts no errors are thrown.

#### Scenario: Multiple cycles without error
- **WHEN** the test registers and unregisters the same shortcut 3 times
- **THEN** no errors are thrown and `isRegistered` returns `false` after the final unregister

### Requirement: Manual test — shortcut trigger callback
The test suite SHALL include a manual test that registers `CommandOrControl+Shift+T` with a handler that logs the event. The TestRunner.svelte SHALL have a button that, when clicked, registers the shortcut and displays instructions for the user to press the key combination.

#### Scenario: User presses shortcut
- **WHEN** the user presses Ctrl+Shift+T on a physical keyboard
- **THEN** the handler callback fires and logs the shortcut event

### Requirement: Permission configuration
The capabilities configuration SHALL include `global-shortcut:allow-register`, `global-shortcut:allow-unregister`, `global-shortcut:allow-unregister-all`, and `global-shortcut:allow-is-registered` permissions.

#### Scenario: Permissions allow all commands
- **WHEN** the test app runs
- **THEN** all 4 global-shortcut IPC commands are permitted
