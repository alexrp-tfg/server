# Server

A self-hosted media server built with Rust, designed to be a modern alternative to services like Immich. This repository contains the backend server implementation, while the mobile application is maintained in a separate repository.

## ⚠️ Development Status

This project is currently in early development. Only user management and authentication features are implemented. Media handling capabilities (photo/video upload, albums, sharing) are planned for future releases.

## Features

### Current Features
- ✅ User registration and management
- ✅ JWT-based authentication
- ✅ Role-based access control
- ✅ RESTful API with OpenAPI documentation
- ✅ Database migrations
- ✅ Docker deployment

### Planned Features
- 📋 Photo and video upload
- 📋 Album management
- 📋 Media sharing and permissions
- 📋 Automatic media organization
- 📋 Face recognition and tagging
- 📋 Mobile app synchronization

## Tech Stack

- **Language**: Rust 1.87+
- **Web Framework**: Axum
- **Database**: PostgreSQL with Diesel ORM
- **Authentication**: JWT tokens with Argon2 password hashing
- **Architecture**: Clean Architecture with CQRS pattern
- **API Documentation**: OpenAPI 3.0 (Swagger UI)
- **Containerization**: Docker & Docker Compose

## Quick Start

### Prerequisites

- Docker and Docker Compose
- (For binary deployment) Rust 1.87+, PostgreSQL, diesel_cli

### Docker Deployment (Recommended)

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd server
   ```

2. Start the services:
   ```bash
   docker-compose up -d
   ```
3. Create a .env file from the example env file
4. The server will be available at `http://localhost:8000`
5. API documentation is available at `http://localhost:8000/doc`

The Docker setup includes:
- PostgreSQL database with automatic health checks
- Automatic database migrations
- Admin user creation
- Server startup

### Binary Deployment

#### Requirements
- Rust 1.87 or later
- PostgreSQL 12+
- diesel_cli: `cargo install diesel_cli --no-default-features --features postgres`

#### Installation Steps

1. **Database Setup**:
   ```bash
   # Install and start PostgreSQL
   sudo apt install postgresql postgresql-contrib
   sudo systemctl start postgresql
   
   # Create database and user
   sudo -u postgres createuser -s your_username
   sudo -u postgres createdb your_database
   ```

2. **Environment Configuration**:
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials and JWT secret
   ```

3. **Database Migrations**:
   ```bash
   diesel migration run
   ```

4. **Build and Run**:
   ```bash
   cargo build --release
   ./target/release/server
   ```

   Or use the provided entrypoint script:
   ```bash
   # Copy the binary to your desired location
   cp target/release/server /path/to/deployment/
   cp entrypoint.sh /path/to/deployment/
   chmod +x /path/to/deployment/entrypoint.sh
   
   # Run with automatic migration and setup
   ./entrypoint.sh
   ```

## Development

### Local Development Setup

1. **Install Dependencies**:
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install diesel CLI
   cargo install diesel_cli --no-default-features --features postgres
   
   # Install watchexec for auto-reload (optional)
   cargo install watchexec-cli
   ```

2. **Database Setup**:
   ```bash
   # Start PostgreSQL (or use Docker)
   docker run -d --name postgres \
     -e POSTGRES_USER=user \
     -e POSTGRES_PASSWORD=password \
     -e POSTGRES_DB=mydb \
     -p 5432:5432 \
     postgres:17.5
   ```

3. **Environment Configuration**:
   ```bash
   cp .env.example .env
   # Edit .env with your development settings
   ```

4. **Run Migrations**:
   ```bash
   diesel migration run
   ```

5. **Start Development Server**:
   ```bash
   # With auto-reload
   ./dev.sh
   
   # Or manually
   cargo run --bin server
   ```

### API Documentation

The server provides interactive API documentation via Swagger UI:
- **URL**: `http://localhost:8000/doc`
- **Format**: OpenAPI 3.0
- **Features**: Interactive testing, authentication flows, schema validation

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test users::

# Run integration tests
cargo test --test tests
```

### Test Structure

The project follows a comprehensive testing strategy:
- **Unit Tests**: Domain logic and business rules
- **Integration Tests**: API endpoints and database operations
- **Repository Tests**: Database layer testing with mocks
- **Authentication Tests**: JWT and security testing

## Contributing

We welcome contributions! Please follow these guidelines:

### Development Workflow

1. **Branching Strategy**: 
   - We use sprint-based branching
   - Create feature branches from the current sprint branch
   - Branch naming: `feature/description` or `fix/description`

2. **Before Contributing**:
   ```bash
   # Fork the repository and clone your fork
   git clone <your-fork-url>
   cd server

   # Move to the current sprint branch
   git checkout sprint/current-sprint-branch
   
   # Create a feature branch
   git checkout -b feature/your-feature-name
   ```

### Code Standards

1. **Testing Requirements**:
   - All new features MUST include comprehensive tests
   - All existing tests MUST pass before merging
   - Aim for high test coverage of business logic

2. **Code Quality**:
   ```bash
   # Format code
   cargo fmt
   
   # Run linting
   cargo clippy
   
   # Ensure tests pass
   cargo test
   ```

3. **Database Changes**:
   - Use Diesel migrations for all schema changes
   - Test migrations both up and down
   - Update repository interfaces as needed

### Commit Guidelines

- Write clear, descriptive commit messages
- Reference issues when applicable
- Keep commits focused and atomic

### Pull Request Process

1. Ensure all tests pass and code is formatted
2. Update documentation if needed
3. Create a pull request to the current sprint branch
4. Request review from maintainers
5. Address feedback and iterate

## Project Structure

```
src/
├── bin/server/          # Application entry point
├── lib/
│   ├── api/            # HTTP layer (routes, middleware)
│   ├── users/          # User domain module
│   │   ├── domain/     # Business logic and entities
│   │   ├── application/ # Use cases (commands/queries)
│   │   ├── infrastructure/ # External concerns (DB, JWT)
│   │   └── interface/   # HTTP controllers
│   ├── shared/         # Shared utilities and interfaces
│   └── persistence/    # Database configuration
tests/                  # Integration and unit tests
migrations/            # Database schema migrations
```

### Architecture Principles

- **Clean Architecture**: Separation of concerns with dependency inversion
- **CQRS**: Command Query Responsibility Segregation
- **Domain-Driven Design**: Rich domain models with clear boundaries
- **Repository Pattern**: Abstracted data access layer

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | - | ✅ |
| `API_PORT` | Server port | `8000` | ❌ |
| `RUST_LOG` | Log level | `info` | ❌ |
| `JWT_SECRET_KEY` | JWT signing secret | - | ✅ |

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPLv3). See the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: Report bugs and request features via GitHub Issues
- **Documentation**: API docs available at `/doc` endpoint
- **Development**: See contributing guidelines above

---

**Note**: This project is under active development. APIs may change and features are still being implemented. Use in production at your own risk.
