# Nonograph TODO Master List

This document tracks all planned improvements for Nonograph. Items are organized by category and priority.

## Security Fixes (CRITICAL - Already Completed)

- [x] Fix critical path traversal vulnerability in post viewing
- [x] Replace insecure DefaultHasher with HMAC-SHA256 for CSRF tokens
- [x] Block path traversal attempts in URL validation
- [x] Add post ID validation function
- [x] Add security tests
- [x] Create SECURITY.md documentation

## JavaScript Removal (CRITICAL - High Threat Model) - COMPLETED

- [x] Remove all JavaScript from home.html
- [x] Ensure basic markdown input works without JS
- [x] Remove slash command menu (JS-dependent feature)
- [x] Remove character counter JavaScript (replaced with static text)
- [x] Remove copy button JavaScript from code blocks
- [x] Test all functionality works without any JavaScript
- [x] Add note in README about zero-JavaScript design
- [x] Update CHANGELOG documenting breaking changes

## Core Features (HIGH PRIORITY - COMPLETED)

- [x] RSS feed at /feed.xml
- [x] Archive page at /archive
- [x] Recent posts on homepage
- [x] RSS autodiscovery meta tag
- [x] Archive and RSS links in homepage sidebar
- [x] Update documentation for new features

## UI Improvements (HIGH PRIORITY) - PARTIALLY COMPLETED

### Editor Improvements - COMPLETED
- [x] Add CSS-only markdown hints in placeholder text
- [x] Add visual markdown reference guide (CSS-only expandable)
- [x] Better character limit indicator (static text)
- [x] Link to full markup guide from quick reference
- [ ] Improve textarea styling and focus states (can still be improved)
- [ ] Improve publish button styling and prominence (can still be improved)
- [ ] Better mobile keyboard experience (can still be improved)

### Typography and Readability - NEEDS WORK
- [ ] Improve post page typography (partially done, can improve)
- [ ] Optimize reading width for long posts
- [ ] Better heading hierarchy
- [ ] Improved line height and spacing
- [ ] Better font stack for readability

### Dark Mode (CSS Only) - COMPLETED
- [x] Implement dark mode using prefers-color-scheme
- [x] Ensure all pages support dark mode
- [x] Test contrast ratios for accessibility
- [x] Dark mode for home page
- [x] Dark mode for post pages
- [x] Dark mode for archive page (uses inline styles, already has dark mode in code)
- [ ] Dark mode for static pages (markup, legal, about, api)

### Mobile Experience
- [ ] Improve mobile touch targets
- [ ] Better mobile spacing and padding
- [ ] Larger text on small screens
- [ ] Optimize mobile form inputs
- [ ] Test on various screen sizes
- [ ] Improve mobile navigation

### Error Pages - COMPLETED
- [x] Design better 404 page
- [x] Add navigation options to error pages
- [x] Consistent error page styling
- [x] Helpful error messages

### Accessibility
- [ ] Improve keyboard navigation
- [ ] Better focus indicators throughout site
- [ ] Screen reader support and ARIA labels
- [ ] Higher contrast ratios
- [ ] Semantic HTML improvements
- [ ] Skip to content link

## Additional Features (MEDIUM PRIORITY)

### Post Tags
- [ ] Add optional tags field to post creation form
- [ ] Store tags in markdown file metadata
- [ ] Create /tag/tagname endpoint
- [ ] Display tags on post pages
- [ ] Tag cloud or list on archive page
- [ ] Tag filtering in archive

### Search Functionality - COMPLETED
- [x] Implement simple text search endpoint
- [x] Search across post titles and content
- [x] Display search results page
- [x] Add search form to homepage or archive
- [x] Rank results by relevance

### Better Static Pages
- [ ] Improve markup guide page design
- [ ] Better about page design
- [ ] Update legal page or remove (not needed for .onion)
- [ ] Consistent styling across static pages

### Post Management
- [ ] Add post editing functionality
- [ ] Edit existing posts at /edit/post-id
- [ ] Show edit history or last modified date
- [ ] Add post deletion capability
- [ ] Confirmation before deletion

## Polish and Refinement (LOW PRIORITY)

### Visual Design
- [ ] Consistent color scheme
- [ ] Better visual hierarchy
- [ ] Improved whitespace usage
- [ ] Consistent border radius and shadows
- [ ] Better hover states

### Performance
- [ ] Optimize CSS delivery
- [ ] Add caching headers
- [ ] Minimize HTML output
- [ ] Lazy load archive page if huge

### Documentation
- [ ] Expand README with more examples
- [ ] Add screenshots to documentation
- [ ] Create CONTRIBUTING.md guide
- [ ] Document all configuration options
- [ ] Create deployment guide for .onion sites

### Testing
- [ ] Add more unit tests
- [ ] Integration tests for new features
- [ ] Test with various markdown edge cases
- [ ] Cross-browser testing (Tor Browser primarily)
- [ ] Accessibility testing

## Nice to Have (OPTIONAL)

- [ ] Related posts feature (show similar posts)
- [ ] Export all posts as zip file
- [ ] Print stylesheet for posts
- [ ] OpenGraph meta tags for better sharing
- [ ] Post view counter (privacy-conscious)
- [ ] RSS feed per tag
- [ ] Atom feed in addition to RSS
- [ ] Multiple author support

## Not Needed (Removed from Consideration)

- ~~Sitemap.xml~~ - Not needed for .onion sites
- ~~Analytics~~ - Against privacy principles
- ~~User accounts~~ - Against anonymity principles
- ~~Comments system~~ - Too much maintenance and moderation
- ~~Social media integration~~ - Against anonymity
- ~~External dependencies~~ - Keep it self-contained

## Implementation Notes

- All features must work without JavaScript
- Prioritize privacy and security over features
- Keep dependencies minimal
- Design for Tor Browser users
- Mobile-first responsive design
- Accessibility is not optional
- Every change must be documented in CHANGELOG.md
