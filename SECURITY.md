# Security Policy

## Supported Versions

Only the latest published release is considered supported for security fixes.

## Reporting a Vulnerability

Please do not open a public GitHub issue for sensitive vulnerabilities.

Preferred process:

1. Contact the repository owner privately if a private channel is available.
2. Share the minimum reproducible details needed to validate the issue.
3. Avoid posting API keys, database files, or personal task data in public.

If no private reporting channel has been configured yet, open a minimal public issue that does not disclose exploit details, and request a private follow-up path.

## Scope Notes

This project stores user data locally and may optionally call a user-configured API endpoint for scoring.

Security-sensitive areas include:

- local API key storage
- outbound API request handling
- import/export parsing
- release artifacts and installer integrity
