## ADDED Requirements

### Requirement: Parse application/x-www-form-urlencoded
The system MUST parse URL-encoded form data (key=value pairs separated by &).

#### Scenario: Parse simple form
- **WHEN** input is "name=John&age=30"
- **THEN** system extracts fields: name="John", age="30"

#### Scenario: Parse form with special characters
- **WHEN** input is "name=John%20Doe&message=Hello%20World"
- **THEN** system URL-decodes and extracts fields: name="John Doe", message="Hello World"

#### Scenario: Parse form with empty values
- **WHEN** input is "name=&value=test"
- **THEN** system extracts fields: name="", value="test"

#### Scenario: Parse form with repeated keys
- **WHEN** input is "color=red&color=blue&color=green"
- **THEN** system extracts fields with multiple values for "color"

### Requirement: Parse multipart/form-data
The system MUST parse multipart form data with boundaries.

#### Scenario: Parse multipart with text field
- **WHEN** input contains multipart body with text field
- **THEN** system extracts field name and value

#### Scenario: Parse multipart with file upload
- **WHEN** input contains multipart body with file upload
- **THEN** system extracts field name, filename, content type, and file data

#### Scenario: Parse multipart with boundary
- **WHEN** Content-Type is "multipart/form-data; boundary=----WebKitFormBoundary"
- **THEN** system uses "----WebKitFormBoundary" as boundary to parse parts

### Requirement: Generate form data
The system MUST generate form data from key-value pairs.

#### Scenario: Generate URL-encoded form
- **WHEN** system has fields: name="John", age="30"
- **THEN** system generates "name=John&age=30"

#### Scenario: Generate URL-encoded form with special characters
- **WHEN** system has fields: name="John Doe"
- **THEN** system generates "name=John%20Doe"

#### Scenario: Generate multipart form
- **WHEN** system has fields and files to upload
- **THEN** system generates multipart body with proper boundaries and headers

### Requirement: Determine content type for form
The system MUST determine the appropriate Content-Type header for form data.

#### Scenario: Determine URL-encoded content type
- **WHEN** form data is application/x-www-form-urlencoded
- **THEN** system returns Content-Type="application/x-www-form-urlencoded"

#### Scenario: Determine multipart content type
- **WHEN** form data contains file uploads
- **THEN** system returns Content-Type="multipart/form-data; boundary=----FormBoundaryXXX"
