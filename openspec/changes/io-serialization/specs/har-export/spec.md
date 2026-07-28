## ADDED Requirements

### Requirement: HAR Log structure

The system SHALL define types for HAR 1.2 Log format.

#### Scenario: HarLog type exists
- **WHEN** the `HarLog` struct is defined
- **THEN** it contains `creator`, `pages`, `entries` fields

#### Scenario: HarEntry type exists
- **WHEN** the `HarEntry` struct is defined
- **THEN** it contains `startedDateTime`, `request`, `response`, `timings` fields

### Requirement: HarExporter

The system SHALL provide a `HarExporter` that converts flows to HAR format.

#### Scenario: HarExporter creates log
- **WHEN** `HarExporter::new()` is called
- **THEN** a new empty HAR log is created

#### Scenario: HarExporter adds entry
- **WHEN** `HarExporter::add_entry(flow)` is called
- **THEN** the flow is converted to a HAR entry and added to the log

#### Scenario: HarExporter exports to JSON
- **WHEN** `HarExporter::export()` is called
- **THEN** the log is serialized to valid HAR JSON

### Requirement: HAR 1.2 compliance

The system SHALL produce HAR 1.2 compliant output.

#### Scenario: Exported JSON matches HAR 1.2 spec
- **WHEN** `HarExporter::export()` is called
- **THEN** the output conforms to HAR 1.2 specification

#### Scenario: Required fields are present
- **WHEN** a HAR entry is exported
- **THEN** it includes `startedDateTime`, `request`, `response` fields

### Requirement: Optional HAR features

The system SHALL support basic HAR features (advanced features are future work).

#### Scenario: Request headers are exported
- **WHEN** a flow is exported to HAR
- **THEN** request headers are included in the entry

#### Scenario: Response status is exported
- **WHEN** a flow is exported to HAR
- **THEN** response status code and reason are included
