## ADDED Requirements

### Requirement: FlowWriter for appending flows

The system SHALL provide a `FlowWriter` that appends serialized flows to a file.

#### Scenario: FlowWriter creates file
- **WHEN** `FlowWriter::new(path)` is called with a new path
- **THEN** the file is created (or truncated if exists)

#### Scenario: FlowWriter appends flow
- **WHEN** `FlowWriter::write(flow)` is called
- **THEN** the flow is appended as a JSON line to the file

#### Scenario: FlowWriter closes properly
- **WHEN** `FlowWriter::close()` is called or dropped
- **THEN** the gzip stream is finalized and file is complete

### Requirement: Gzip compression

The system SHALL use gzip compression for dump files.

#### Scenario: Dump file is gzip compressed
- **WHEN** a flow is written to a `.jsonl.gz` file
- **THEN** the file is valid gzip format

#### Scenario: Gzip file can be read by standard tools
- **WHEN** a dump file is opened with `gunzip` or `zcat`
- **THEN** it produces valid JSON lines

### Requirement: FlowReader for reading flows

The system SHALL provide a `FlowReader` that reads flows from a file.

#### Scenario: FlowReader opens file
- **WHEN** `FlowReader::new(path)` is called
- **THEN** the gzip stream is initialized for reading

#### Scenario: FlowReader reads flow
- **WHEN** `FlowReader::read_next()` is called
- **THEN** the next flow is deserialized and returned

#### Scenario: FlowReader detects end of file
- **WHEN** `FlowReader::read_next()` is called after all flows
- **THEN** it returns `None` or an end-of-stream error

### Requirement: Streaming I/O

The system SHALL use buffered I/O for efficient reading and writing.

#### Scenario: FlowWriter uses BufWriter
- **WHEN** `FlowWriter` is created
- **THEN** it uses `BufWriter` internally for buffered writes

#### Scenario: FlowReader uses line-by-line reading
- **WHEN** `FlowReader` reads flows
- **THEN** it reads one JSON line at a time (streaming)
