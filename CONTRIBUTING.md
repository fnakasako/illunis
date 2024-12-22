# Contributing to Sovereign Attention Protocol

We're excited that you're interested in contributing to SAP! This document provides guidelines and instructions for contributing.

## Core Values

When contributing, please keep in mind our core values:

1. **User Sovereignty**: All features must preserve user control and privacy
2. **Local First**: Core functionality must work without external dependencies
3. **Progressive Enhancement**: Features should layer from simple to complex
4. **Security**: Privacy and security are non-negotiable requirements

## Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/sovereign-attention-protocol.git
   cd sovereign-attention-protocol
   ```
3. Install dependencies:
   ```bash
   npm install
   ```
4. Create a branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Project Structure

```
src/
├── core/                 # Core processing layer
│   ├── AttentionTracker # Attention metrics tracking
│   ├── ContentFilter    # Content filtering system
│   ├── DataStore        # Local data persistence
│   └── LocalProcessor   # Main processing coordinator
├── platform/            # Platform integration layer
├── federation/          # Optional federated features
└── cli/                 # Command-line interface
```

## Coding Standards

- Use ES modules (import/export)
- Follow functional programming principles where appropriate
- Write tests for new features
- Document public APIs
- Use TypeScript for type safety
- Follow privacy-by-design principles

## Testing

```bash
# Run all tests
npm test

# Run specific test suite
npm test -- --grep "AttentionTracker"

# Run with coverage
npm run test:coverage
```

## Pull Request Process

1. Update documentation for new features
2. Add tests for new functionality
3. Ensure all tests pass
4. Update CHANGELOG.md
5. Submit PR with clear description

## Feature Guidelines

### Security Features
- Must be opt-in when involving network operations
- Must have clear privacy implications documented
- Must support offline operation

### Platform Integration
- Must be modular and optional
- Must normalize data to standard formats
- Must respect rate limits and platform ToS

### Federation Features
- Must be optional and not impact core functionality
- Must implement differential privacy
- Must have clear trust model documented

## Documentation

- Update README.md for user-facing changes
- Update API documentation for interface changes
- Include JSDoc comments for public methods
- Document privacy implications of features

## Questions?

Feel free to open an issue for:
- Feature proposals
- Architecture discussions
- Security considerations
- Implementation questions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
