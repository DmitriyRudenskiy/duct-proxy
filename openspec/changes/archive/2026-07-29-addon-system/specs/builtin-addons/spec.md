## ADDED Requirements

### Requirement: ModifyHeaders addon
The system SHALL provide a built-in addon to modify HTTP headers.

#### Scenario: Add header
- **WHEN** `ModifyHeaders::add(name, value)` is configured
- **THEN** the header is added to all matching requests/responses

#### Scenario: Set header
- **WHEN** `ModifyHeaders::set(name, value)` is configured
- **THEN** the header value is replaced if it exists, otherwise added

#### Scenario: Remove header
- **WHEN** `ModifyHeaders::remove(name)` is configured
- **THEN** the header is removed from all matching requests/responses

#### Scenario: Filter by regex
- **WHEN** `ModifyHeaders::filter(regex)` is configured
- **THEN** only flows matching the regex pattern are modified

### Requirement: ModifyBody addon
The system SHALL provide a built-in addon to modify HTTP request/response bodies.

#### Scenario: Replace body content
- **WHEN** `ModifyBody::replace(pattern, replacement)` is configured
- **THEN** all occurrences of pattern in the body are replaced

#### Scenario: Replace body by regex
- **WHEN** `ModifyBody::replace_regex(regex, replacement)` is configured
- **THEN** all regex matches in the body are replaced

#### Scenario: Filter by content type
- **WHEN** `ModifyBody::filter(content_type)` is configured
- **THEN** only flows with matching Content-Type are modified

### Requirement: Block addon
The system SHALL provide a built-in addon to block requests matching a filter.

#### Scenario: Block by URL pattern
- **WHEN** `Block::url_pattern(regex)` is configured
- **THEN** requests matching the URL pattern receive a 403 response

#### Scenario: Block by header
- **WHEN** `Block::header(name, value)` is configured
- **THEN** requests with matching header receive a 403 response

#### Scenario: Block by source IP
- **WHEN** `Block::source_ip(ip_range)` is configured
- **THEN** requests from matching IP ranges receive a 403 response

### Requirement: Filter addon
The system SHALL provide a built-in addon to filter flows based on expressions.

#### Scenario: Filter by URL
- **WHEN** `Filter::url(regex)` is configured
- **THEN** only flows matching the URL pattern are passed to other addons

#### Scenario: Filter by method
- **WHEN** `Filter::method(method)` is configured
- **THEN** only flows with matching HTTP method are passed

#### Scenario: Filter by header
- **WHEN** `Filter::header(name, value)` is configured
- **THEN** only flows with matching header are passed

#### Scenario: Combine filters with AND
- **WHEN** multiple filters are combined with `.and()`
- **THEN** all filters must match for the flow to pass

#### Scenario: Combine filters with OR
- **WHEN** multiple filters are combined with `.or()`
- **THEN** any filter can match for the flow to pass
