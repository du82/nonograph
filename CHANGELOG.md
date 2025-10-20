# Changelog

All notable changes to Nonograph will be documented in this file.

## [Unreleased]

### Breaking Changes

#### 100% JavaScript-Free Design

All JavaScript has been removed from Nonograph to support users with extremely high threat models. The site now works perfectly without any JavaScript enabled, making it ideal for Tor Browser users with maximum security settings.

Changes made:
- Removed all JavaScript from homepage (home.html)
- Removed all JavaScript from post pages (post.html)
- Replaced slash command editor menu with simple markdown placeholder hints
- Added CSS-only collapsible markdown formatting guide
- Removed JavaScript-dependent character counter (replaced with static limit text)
- Removed copy-to-clipboard buttons from code blocks (not possible without JS)
- Secret text reveal feature now works with CSS-only hover (no click required)

This is a breaking change if users relied on the JavaScript editor features, but significantly improves security and privacy for high-threat-model users.

### New Features

#### CSS-Only Dark Mode

Implemented automatic dark mode that respects the user's system preferences using the `prefers-color-scheme` CSS media query. No JavaScript or manual toggle required.

Benefits:
- Automatically switches based on system settings
- Works in Tor Browser with privacy.resistFingerprinting enabled
- Zero performance overhead
- No storage or cookies needed
- Applies to homepage, post pages, and archive

Colors optimized for readability in both light and dark modes with proper contrast ratios for accessibility.

#### Markdown Formatting Help (CSS-Only)

Added a collapsible markdown formatting reference guide on the homepage that works entirely with CSS using the checkbox hack. Users can click to expand inline examples of all supported markdown syntax.

Features:
- No JavaScript required
- Quick reference for common markdown syntax
- Link to full markup guide for detailed documentation
- Collapses by default to avoid clutter

#### Recent Posts on Homepage

The homepage now displays the 5 most recent posts below the submission form. This makes the site feel more active and helps visitors discover new content immediately when they arrive. The list shows post titles, authors (if provided), and publication dates, with direct links to each post.

#### Archive Page

Added a dedicated archive page at `/archive` that lists all published posts in chronological order (newest first). This provides a simple way to browse through all content on the site without needing to know specific post URLs. Each entry shows the post title, author, and date. The archive page is accessible from the homepage sidebar.

Benefits:
- Makes old content discoverable
- No JavaScript required, just server-side rendering
- Simple chronological listing
- Mobile responsive design

#### RSS Feed

Implemented a full RSS 2.0 feed at `/feed.xml` that includes the 20 most recent posts. This is particularly valuable for anonymous publishing because:
- Readers can follow the site without visiting directly
- Works perfectly with Tor browser RSS readers
- Standard RSS format compatible with all feed readers
- Includes RSS autodiscovery meta tag on homepage

The feed includes post titles, publication dates, authors, and direct links to posts. The RSS link is available in the homepage sidebar, and browsers with RSS support will automatically detect the feed.

#### Simple Search Functionality

Added a basic search feature at `/search` that allows users to find posts by searching titles, authors, and content. Search results are ranked by relevance with title matches weighted higher than content matches.

Features:
- Search across all post titles, authors, and content
- Relevance-based ranking (title > author > content)
- Minimum 2 character search query
- Results sorted by relevance, then by date
- Mobile-responsive design
- Dark mode support
- No JavaScript required
- Search link in homepage sidebar

The search is implemented as a simple grep-style search through markdown files, which works well for small to medium-sized sites without needing a search index or database.

#### Improved 404 Error Page

Completely redesigned the 404 error page with:
- Modern, centered layout
- Large, clear 404 heading
- Helpful navigation options (Home and Archive)
- Dark mode support
- Mobile responsive
- Consistent branding with the rest of the site

The new error page provides a better user experience when accessing non-existent posts and helps users navigate back to useful parts of the site.

### Security Fixes

#### Critical Path Traversal Vulnerability Fixed

Fixed a critical security vulnerability where attackers could read arbitrary files from the server by manipulating post IDs. The application now validates all post IDs to ensure they only contain safe characters (alphanumeric, hyphens, and underscores). This prevents directory traversal attacks like requesting `/../../../etc/passwd`.

Changes made:
- Added `is_valid_post_id()` validation function
- Post IDs are now limited to 256 characters maximum
- Characters like `..`, `/`, and `\` are blocked
- Validation is applied to all post viewing endpoints
- Static page names are now validated against a whitelist

#### CSRF Token Security Improvement

Replaced the insecure `DefaultHasher` with cryptographically secure HMAC-SHA256 for CSRF token generation and validation. The previous implementation used a non-cryptographic hash function that could potentially be predicted or forged by attackers.

Changes made:
- CSRF tokens now use HMAC-SHA256 instead of DefaultHasher
- Added a secure 512-bit secret key that persists for the application lifetime
- Tokens are validated using constant-time comparison to prevent timing attacks
- Token expiration remains at 24 hours as before

#### URL Path Traversal Protection

Enhanced URL validation to prevent path traversal attempts in image and link URLs. The system now blocks any URL containing `..` to prevent users from embedding links that attempt directory traversal.

Changes made:
- `is_safe_url()` function now rejects URLs containing `..`
- Applies to both image embeds and hyperlinks
- Relative URLs without traversal attempts are still allowed

### Dependencies Added

- `hmac = "0.12"` - HMAC implementation for secure token generation
- `sha2 = "0.10"` - SHA-256 hashing for HMAC
- `hex = "0.4"` - Hexadecimal encoding for token formatting

### Testing

Added comprehensive security tests:
- `test_post_id_validation_security()` - Validates path traversal blocking
- `test_csrf_token_generation_and_validation()` - Tests token security
- `test_csrf_secret_consistency()` - Verifies secret key generation
- Enhanced URL validation tests for path traversal protection

## [0.1.0] - Initial Release

Initial release of Nonograph, an anonymous publishing platform designed for privacy and simplicity.
