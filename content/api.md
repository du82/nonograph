October 05, 2025

# API Documentation
Nonograph provides a simple HTTP API for programmatically publishing articles.

## Publishing a Post
**Endpoint:** `POST /create`
**Content-Type:** `application/x-www-form-urlencoded`

### Parameters
| Field | Type | Required | Max Length | Description |
|-------|------|----------|------------|-------------|
| `title` | string | Yes | 128 chars | Article title |
| `content` | string | Yes | 32,000 chars | Article content in markdown |
| `alias` | string | No | 32 chars | Alias name (optional) |

### Example Request
```bash
curl -X POST http://localhost:8000/create \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "title=My API Article" \
  -d "alias=API User" \
  -d "content=This article was created via the API!

## Features

- **Bold text** and *italic text*
- [Links](https://example.com) open in new tabs
- Code blocks with syntax highlighting

\`\`\`python
def hello_world():
    print('Hello from the API!')
\`\`\`

| Feature | Status |
|---------|--------|
| API | ✓ Working |
| Markdown | ✓ Supported |"
```

### Successful Response
**Status:** `302 Found`
**Location:** `/{post-id}`

The API redirects to the published article URL. Extract the post ID from the Location header.

### Error Responses
**Status:** `302 Found` (redirect to home with error parameter)

| Error Parameter | Description |
|----------------|-------------|
| `?error=title_required` | Title is empty |
| `?error=content_required` | Content is empty |
| `?error=title_too_long` | Title exceeds 128 characters |
| `?error=content_too_long` | Content exceeds 32,000 characters |
| `?error=alias_too_long` | Alias exceeds 32 characters |
| `?error=no_available_slots` | No available post ID slots (rare) |

## Post URLs
Published posts are accessible at: `/{post-id}`

Post IDs are generated from the title and date: `title-slug-mm-dd-yyyy`

## Raw Markdown Access
Access the original markdown by appending `.md`: `/{post-id}.md`

## Supported Content
All standard Nonograph markdown features are supported:

- **Text formatting:** bold, italic, underline, strikethrough, superscript
- **Links:** `[text](url)` and `[bare-url]`
- **Code blocks:** Fenced blocks with 180+ language support
- **Tables:** GitHub-style markdown tables
- **Media:** Auto-embedded images and videos from URLs
- **Secret text:** `#hidden text#` - click to reveal

## Rate Limiting
No rate limiting is currently implemented. Please use responsibly.

## Examples
### Python

```python
import requests

data = {
    'title': 'API Test Article',
    'alias': 'Python Script',
    'content': '''# Hello from Python!

This article was created programmatically.

## Code Example

\`\`\`python
import requests
print("API publishing works!")
\`\`\`

[Visit the docs](https://example.com)'''
}

response = requests.post('http://localhost:8000/create', data=data)
if response.status_code == 200:
    post_url = response.url
    print(f"Published: {post_url}")
```

### JavaScript (Node.js)
```javascript
const axios = require('axios');

const data = new URLSearchParams({
    title: 'API Test Article',
    alias: 'Node.js Script',
    content: `# Hello from JavaScript!

This article was created with Node.js.

## Features
- API integration
- Markdown support
- [External links](https://nodejs.org)`
});

axios.post('http://localhost:8000/create', data)
    .then(response => {
        console.log('Published:', response.request.res.responseUrl);
    })
    .catch(error => {
        console.error('Error:', error.response?.status);
    });
```

### Shell Script
```bash
#!/bin/bash

TITLE="Daily Report $(date +%Y-%m-%d)"
ALIAS="Automation"
CONTENT="# Daily System Report

Generated at: $(date)

## Status
- System: ✓ Online
- Services: ✓ Running

## Metrics
| Metric | Value |
|--------|-------|
| CPU | 15% |
| Memory | 2.1GB |

[Dashboard](https://monitoring.example.com)"

curl -X POST http://localhost:8000/create \
  --data-urlencode "title=$TITLE" \
  --data-urlencode "alias=$ALIAS" \
  --data-urlencode "content=$CONTENT" \
  --location \
  --silent \
  --write-out "Published: %{url_effective}\n"
```

## Notes
- Posts are stored as markdown files and cached in memory
- No authentication required - posts are public once published
- Content is sanitized for security but preserves intended formatting
- Images and videos must be hosted externally - only URLs are stored
- Post IDs are generated deterministically from title and date