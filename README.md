# Blog API Documentation

This document provides comprehensive documentation for the Blog API endpoints, including data types, request/response formats, and error handling.

## Base URL

```
http://localhost:8080
```

## Authentication

Currently, the API does not implement authentication. All endpoints are publicly accessible.

## Data Types

### Post Category
Posts can belong to one of the following categories:
- `blog`
- `art`
- `reading`

### Post
```typescript
{
  id: number;
  category: "blog" | "art" | "reading";
  title: string;
  slug: string;
  content: string;
  description: string;
  image_url?: string;
  external_url?: string;
  published: boolean;
  created_at: string;  // ISO 8601 datetime
  updated_at: string;  // ISO 8601 datetime
}
```

### Tag
```typescript
{
  id: number;
  name: string;
  created_at: string;  // ISO 8601 datetime
}
```

### TagWithPostCount
```typescript
{
  id: number;
  name: string;
  created_at: string;  // ISO 8601 datetime
  post_count: number;
}
```

## Error Handling

The API returns appropriate HTTP status codes along with error messages in the following format:

```typescript
{
  message: string;
}
```

Common error status codes:
- `400 Bad Request`: Invalid input data
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource already exists (e.g., duplicate slug or tag name)
- `500 Internal Server Error`: Server-side error

## Endpoints

### Posts

#### List Posts
```http
GET /posts?category=blog&published_only=true&limit=20&offset=0
```

Query Parameters:
- `category` (optional): Filter by post category
- `published_only` (optional): If true, returns only published posts
- `limit` (optional): Maximum number of posts to return (default: 20, max: 100)
- `offset` (optional): Number of posts to skip for pagination

Response: `200 OK`
```json
[
  {
    "id": 1,
    "category": "blog",
    "title": "My First Post",
    "slug": "my-first-post",
    "content": "Content goes here...",
    "description": "Brief description",
    "image_url": null,
    "external_url": null,
    "published": true,
    "created_at": "2024-01-11T10:00:00Z",
    "updated_at": "2024-01-11T10:00:00Z"
  }
]
```

#### Create Post
```http
POST /posts
```

Request Body:
```json
{
  "category": "blog",
  "title": "My New Post",
  "slug": "my-new-post",
  "content": "Post content goes here...",
  "description": "Brief description",
  "image_url": null,
  "external_url": null,
  "published": false
}
```

Response: `200 OK`
Returns the created post object.

#### Get Post by ID
```http
GET /posts/by-id/{id}
```

Response: `200 OK`
Returns the post object.

#### Get Post by Slug
```http
GET /posts/by-slug/{slug}
```

Response: `200 OK`
Returns the post object.

#### Update Post
```http
PUT /posts
```

Request Body:
```json
{
  "id": 1,
  "category": "blog",
  "title": "Updated Title",
  "slug": "updated-slug",
  "content": "Updated content",
  "description": "Updated description",
  "image_url": null,
  "external_url": null,
  "published": true
}
```

Response: `200 OK`
Returns the updated post object.

#### Patch Post
```http
PATCH /posts
```

Request Body (all fields optional except id):
```json
{
  "id": 1,
  "title": "New Title",
  "published": true
}
```

Response: `200 OK`
Returns the updated post object.

#### Delete Post
```http
DELETE /posts/{id}
```

Response: `204 No Content`

### Tags

#### List Tags
```http
GET /tags?include_post_count=true
```

Query Parameters:
- `include_post_count` (optional): If true, includes the count of posts for each tag

Response: `200 OK`
```json
[
  {
    "id": 1,
    "name": "rust",
    "created_at": "2024-01-11T10:00:00Z",
    "post_count": 5
  }
]
```

#### Create Tag
```http
POST /tags
```

Request Body:
```json
{
  "name": "rust"
}
```

Response: `200 OK`
Returns the created tag object.

#### Get Tag by ID
```http
GET /tags/{id}
```

Response: `200 OK`
Returns the tag object.

#### Get Tag by Name
```http
GET /tags/by-name/{name}
```

Response: `200 OK`
Returns the tag object.

#### Update Tag
```http
PUT /tags/{id}
```

Request Body:
```json
{
  "name": "new-name"
}
```

Response: `200 OK`
Returns the updated tag object.

#### Delete Tag
```http
DELETE /tags/{id}
```

Response: `204 No Content`

### Post-Tag Relationships

#### Get Post Tags
```http
GET /posts/{post_id}/tags
```

Response: `200 OK`
```json
[
  {
    "id": 1,
    "name": "rust",
    "created_at": "2024-01-11T10:00:00Z"
  }
]
```

#### Add Tag to Post
```http
PUT /posts/{post_id}/tags/{tag_id}
```

Response: `204 No Content`

#### Remove Tag from Post
```http
DELETE /posts/{post_id}/tags/{tag_id}
```

Response: `204 No Content`

## Validation Rules

### Posts
- Title cannot be empty
- Content cannot be empty
- Slug must be URL-friendly (alphanumeric characters and hyphens only)
- Slug cannot start or end with a hyphen
- Post ID must be positive
- Limit must be between 1 and 100 for listing posts

### Tags
- Name cannot be empty
- Name must be 50 characters or less
- Name can only contain alphanumeric characters, spaces, hyphens, underscores, and plus signs
- Name must be unique

## CORS

The API supports Cross-Origin Resource Sharing (CORS) and allows:
- All origins
- All methods
- All headers

## Example Usage

Here's an example of how to create a new post and add tags to it:

```javascript
// Create a new post
const post = await fetch('http://localhost:8080/posts', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    category: 'blog',
    title: 'Getting Started with Rust',
    slug: 'getting-started-with-rust',
    content: 'Content goes here...',
    description: 'A beginner-friendly guide to Rust programming',
    published: true
  })
});

const postData = await post.json();

// Create a new tag
const tag = await fetch('http://localhost:8080/tags', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    name: 'rust'
  })
});

const tagData = await tag.json();

// Add tag to post
await fetch(`http://localhost:8080/posts/${postData.id}/tags/${tagData.id}`, {
  method: 'PUT'
});

// Get all tags for the post
const postTags = await fetch(`http://localhost:8080/posts/${postData.id}/tags`);
const postTagsData = await postTags.json();
```
