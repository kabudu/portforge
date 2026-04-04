# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | ✅ Current release |

## Reporting a Vulnerability

If you discover a security vulnerability in PortForge, please report it responsibly.

### How to Report

1. **Do NOT open a public GitHub issue** for security vulnerabilities.
2. Send an email to **security@kabudu.dev** (or use [GitHub Security Advisories](https://github.com/kabudu/portforge/security/advisories/new)).
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment** within 48 hours of your report.
- **Status update** within 7 days with our assessment.
- **Fix timeline** communicated once the issue is confirmed.
- **Credit** in the release notes (unless you prefer to remain anonymous).

## Security Considerations

PortForge operates with the following security model:

### Process Management
- `portforge kill` sends signals to processes on the local machine.
- The `clean` command only targets orphaned/zombie processes by default.
- All destructive actions require explicit confirmation or `--force` flag.
- Dry-run mode (`--dry-run`) is available for safe preview.

### Network
- Health checks connect only to `127.0.0.1` (localhost).
- The web dashboard (`portforge serve`) binds to `127.0.0.1` by default.
- No data is sent to external servers.
- Docker communication uses the local Docker socket.

### File Access
- Configuration is read from `~/.config/portforge.toml` only.
- No files are written except the config file (via `init-config`).
- Process information is read from system APIs (sysinfo, /proc, etc.).

### Dependencies
- We minimize dependencies and prefer well-maintained Rust crates.
- `cargo audit` is run regularly to check for known vulnerabilities.
- TLS is handled via `rustls` (no OpenSSL dependency).

## Best Practices for Users

1. **Don't expose the web dashboard** to the public internet without authentication.
2. **Review `portforge clean` output** with `--dry-run` before executing.
3. **Keep PortForge updated** to receive security patches.
