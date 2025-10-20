# Security

## Security Philosophy

Nonograph is designed with privacy and security as core principles. Since this is a self-hosted anonymous publishing platform, security measures are implemented at multiple layers to protect both the operator and users.

## Implemented Security Measures

### Input Validation

All user input is validated and sanitized:

- Post IDs are validated to only contain alphanumeric characters, hyphens, and underscores
- Directory traversal attempts (using `..`, `/`, or `\`) are blocked
- Post titles are limited to 128 characters
- Author names are limited to 32 characters
- Content is limited to 128,000 characters
- Form data is limited to 512KB

### XSS Protection

Cross-site scripting attacks are prevented through:

- HTML sanitization using the Ammonia library
- All user-provided text is stripped of dangerous HTML tags
- Script tags, event handlers, and javascript: URLs are blocked
- Content Security is enforced on all rendered content

### CSRF Protection

Cross-site request forgery protection is enabled by default:

- CSRF tokens are generated using HMAC-SHA256
- Tokens expire after 24 hours
- Each token includes a cryptographically secure random component
- Token validation uses constant-time comparison to prevent timing attacks
- Can be configured in Config.toml (though disabling is not recommended)

### URL Security

Embedded URLs in posts are validated:

- javascript:, data:, and file: protocols are blocked
- Private IP ranges (10.x, 192.168.x, 127.x, 169.254.x) are blocked
- Localhost references are blocked to prevent SSRF attacks
- Path traversal attempts in URLs are blocked
- URL length is limited to 4096 characters

### Path Traversal Prevention

File operations are protected against directory traversal:

- Post IDs cannot contain directory separators
- Static page names are validated against a whitelist
- All file paths are constructed safely
- File reads are restricted to the content directory

### No Authentication System

Nonograph intentionally has no user accounts or authentication:

- No passwords to leak or crack
- No session management vulnerabilities
- No user enumeration possible
- No credential stuffing or brute force risks

## Security Configuration

### Config.toml Settings

```toml
[security]
# Maximum URL length for link processing
max_url_length = 4096

# Enable/disable external link security attributes
external_link_security = true

# Enable/disable CSRF protection
# For production deployments, this should always be 'true'
csrf_protection_enabled = true
```

### Recommendations for Production

1. Always keep `csrf_protection_enabled = true`
2. Run Nonograph behind a reverse proxy (nginx, Apache)
3. Use HTTPS/TLS when running on clearnet
4. Keep the system and dependencies updated
5. Limit file system permissions for the Nonograph process
6. Run Nonograph as a non-privileged user
7. Consider rate limiting at the reverse proxy level

## Known Limitations

### No Built-in Rate Limiting

Nonograph does not include built-in rate limiting. For production deployments, implement rate limiting at the reverse proxy or firewall level to prevent:

- Post spam
- Storage exhaustion
- Denial of service attempts

### No Content Moderation

There is no built-in content moderation system. As a self-hosted platform, the operator is responsible for monitoring published content and removing unwanted posts manually if needed.

### File-based Storage

Posts are stored as markdown files on disk. While simple and reliable, this means:

- No automatic backup system
- No built-in replication
- Storage is limited by disk space
- Operators should implement their own backup strategy

## Reporting Security Issues

If you discover a security vulnerability in Nonograph, please report it responsibly:

1. Do not open a public GitHub issue
2. Email the details privately to the maintainer
3. Include steps to reproduce the vulnerability
4. Allow time for a fix to be developed before public disclosure

## Security Audit History

- Initial security audit completed: January 2025
  - Fixed critical path traversal vulnerability
  - Upgraded CSRF token generation to HMAC-SHA256
  - Enhanced URL validation to prevent path traversal

## Dependencies and Supply Chain

Nonograph uses minimal dependencies to reduce supply chain risk:

- rocket (web framework)
- serde (serialization)
- chrono (date/time handling)
- rand (random number generation)
- ammonia (HTML sanitization)
- toml (configuration parsing)
- hmac (HMAC implementation)
- sha2 (SHA-256 hashing)
- hex (hexadecimal encoding)

All dependencies are from well-maintained crates with good security track records.
