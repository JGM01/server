-- Add migration script here
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE posts (
    id INTEGER PRIMARY KEY,
    type TEXT NOT NULL CHECK(type IN ('blog', 'art', 'reading')),
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    description TEXT NOT NULL,
    image_url TEXT,
    external_url TEXT,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Junction table for posts and tags
CREATE TABLE post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    -- Make the combination unique to prevent duplicate tags on a post
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);


-- Index for faster slug lookups since we'll use these in URLs
CREATE INDEX idx_posts_slug ON posts(slug);
-- Index for filtering by type and ordering by date
CREATE INDEX idx_posts_type_date ON posts(type, created_at DESC);
-- Index for finding published posts quickly
CREATE INDEX idx_posts_published ON posts(published, created_at DESC);
