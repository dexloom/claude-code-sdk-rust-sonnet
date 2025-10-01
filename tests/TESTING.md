# Testing Guide for Claude Agent SDK

This document describes the testing strategy and how to run tests for the Rust SDK.

## Test Organization

### Unit Tests
Located in `tests/` directory:
- `test_types.rs` - Type definitions and serialization
- `test_errors.rs` - Error handling
- `test_message_parser.rs` - Message parsing
- `test_transport.rs` - Transport layer with mocks

### Integration Tests
- `integration_tests.rs` - End-to-end workflows with mock transport

### Example Tests
- Located in `examples/` - Demonstrate real usage patterns
- Require Claude CLI to be installed for execution

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test File
```bash
cargo test --test test_message_parser
cargo test --test test_errors
cargo test --test test_types
cargo test --test integration_tests
```

### Specific Test Function
```bash
cargo test test_parse_text_content_block
cargo test test_cli_not_found_error
```

### With Output
```bash
cargo test -- --nocapture
```

### With Specific Number of Threads
```bash
cargo test -- --test-threads=1
```

## Test Coverage

### Unit Test Coverage

#### Types Module (`test_types.rs`)
- ✅ ClaudeAgentOptions default values
- ✅ ClaudeAgentOptions builder pattern
- ✅ ClaudeAgentOptions clone and debug
- ✅ Permission mode constants
- ✅ Hook event constants
- ✅ SettingSource serialization
- ✅ SystemPrompt variants
- ✅ AgentDefinition
- ✅ PermissionRuleValue
- ✅ PermissionResult variants
- ✅ McpServerConfig variants (Stdio, SSE, HTTP, SDK)
- ✅ ContentBlock variants
- ✅ HookMatcher debug and clone
- ✅ Tool permission context
- ✅ SDK control protocol types

#### Errors Module (`test_errors.rs`)
- ✅ CLINotFound error
- ✅ CLIConnection error
- ✅ Process error with exit code and stderr
- ✅ MessageParse error with data
- ✅ ControlProtocol error
- ✅ Transport error
- ✅ InvalidConfig error
- ✅ Timeout error
- ✅ JSONDecode error (from serde)
- ✅ IO error (from std::io)
- ✅ Result type alias
- ✅ Error chaining
- ✅ Error display formatting
- ✅ Send + Sync trait bounds

#### Message Parser (`test_message_parser.rs`)
- ✅ Parse text content block
- ✅ Parse thinking block
- ✅ Parse tool use block
- ✅ Parse tool result block
- ✅ Parse user message
- ✅ Parse system message
- ✅ Parse result message
- ✅ Parse stream event
- ✅ Parse multiple content blocks
- ✅ Invalid message type handling
- ✅ Missing required field handling
- ✅ Invalid content block type handling

#### Transport Layer (`test_transport.rs`)
- ✅ Mock transport connect
- ✅ Mock transport write
- ✅ Write before connect (error case)
- ✅ Read messages
- ✅ Close transport
- ✅ Multiple writes
- ✅ End input
- ✅ JSON message handling
- ✅ Transport trait bounds (Send)
- ✅ Complete lifecycle

### Integration Tests (`integration_tests.rs`)
- ✅ Query with mock transport
- ✅ Streaming conversation
- ✅ Tool use workflow
- ✅ Error message handling
- ✅ System message handling
- ✅ Options configuration
- ✅ Concurrent message processing

## Test Statistics

```
Total Test Files: 5
Total Test Functions: ~80
Coverage Areas:
  - Types & Serialization: ~35 tests
  - Error Handling: ~15 tests
  - Message Parsing: ~15 tests
  - Transport Layer: ~10 tests
  - Integration: ~10 tests
```

## Mock Transport

The `MockTransport` struct in `test_transport.rs` provides a test double for the Transport trait:

```rust
let messages = vec![
    json!({"type": "assistant", "message": {...}}),
    json!({"type": "result", ...}),
];

let transport = MockTransport::new(messages);
// Use in tests...
```

Features:
- Pre-configured message sequences
- Tracks written data
- Connection state management
- Implements full Transport trait

## Writing New Tests

### Unit Test Template

```rust
#[test]
fn test_feature_name() {
    // Arrange
    let input = create_test_input();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected_value);
}
```

### Async Test Template

```rust
#[tokio::test]
async fn test_async_feature() {
    // Arrange
    let input = create_test_input();

    // Act
    let result = async_function(input).await.unwrap();

    // Assert
    assert_eq!(result, expected_value);
}
```

### Integration Test Template

```rust
#[tokio::test]
async fn test_workflow_name() {
    // Create mock transport with messages
    let messages = vec![...];
    let transport = MockTransport::new(messages);

    // Create query/client
    let mut query = Query::new(transport, ...);

    // Execute workflow
    let mut stream = query.receive_messages();

    // Verify behavior
    while let Some(msg) = stream.next().await {
        // Assertions...
    }
}
```

## Continuous Integration

### GitHub Actions (Future)

```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo test --all-features
```

### Pre-commit Checks

```bash
#!/bin/bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Test-Driven Development

When adding new features:

1. Write failing test first
2. Implement minimum code to pass
3. Refactor while keeping tests green
4. Add edge case tests

Example workflow:
```bash
# 1. Write test
cargo test test_new_feature  # Fails

# 2. Implement
# ... edit code ...

# 3. Verify
cargo test test_new_feature  # Passes

# 4. Full suite
cargo test                   # All pass
```

## Known Limitations

### Not Tested (Requires CLI)
- Actual subprocess spawning
- Real CLI communication
- Stderr callbacks with real process
- Permission callbacks with real prompts
- MCP server integration
- Hook execution

### Future Testing Needs
- Property-based testing with `proptest`
- Fuzzing with `cargo-fuzz`
- Benchmark tests with `criterion`
- Memory leak detection with `valgrind`
- Performance regression tests

## Benchmarking (Future)

```bash
cargo bench
```

Example benchmark:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_benchmark(c: &mut Criterion) {
    let data = json!({...});
    c.bench_function("parse_message", |b| {
        b.iter(|| parse_message(black_box(data.clone())))
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
```

## Debugging Tests

### Print Debug Output
```bash
cargo test -- --nocapture
```

### Run Single Test with Logging
```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Test with Backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Use `dbg!` Macro in Tests
```rust
#[test]
fn debug_test() {
    let value = compute_something();
    dbg!(&value);  // Prints value during test
    assert_eq!(value, expected);
}
```

## Best Practices

1. **Test Names**: Use descriptive names starting with `test_`
2. **Arrange-Act-Assert**: Follow AAA pattern
3. **One Assertion**: Focus each test on one thing
4. **Fast Tests**: Keep unit tests under 100ms
5. **Deterministic**: No flaky tests, no random data
6. **Isolated**: Tests shouldn't depend on each other
7. **Readable**: Tests are documentation
8. **Mock External**: Use mocks for I/O and network

## Contributing Tests

When contributing:
- Add tests for new features
- Update tests for changed behavior
- Ensure all tests pass: `cargo test`
- Check formatting: `cargo fmt`
- Check lints: `cargo clippy`
- Maintain >80% coverage for new code

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [tokio Testing](https://tokio.rs/tokio/topics/testing)
- [async-trait Documentation](https://docs.rs/async-trait/)
- [assert_matches Crate](https://docs.rs/assert_matches/)
