# 1. Create a blog post
echo "Creating a blog post..."
curl -X POST http://localhost:8080/posts \
  -H "Content-Type: application/json" \
  -d '{
    "category": "blog",
    "title": "My First Post",
    "slug": "my-first-post",
    "content": "This is the content of my first post.",
    "description": "A brief description of my first post",
    "published": true
  }'

# 2. Create some tags
echo -e "\n\nCreating tags..."
curl -X POST http://localhost:8080/tags \
  -H "Content-Type: application/json" \
  -d '{"name": "rust"}'

curl -X POST http://localhost:8080/tags \
  -H "Content-Type: application/json" \
  -d '{"name": "programming"}'

# 3. List all tags
echo -e "\n\nListing all tags..."
curl http://localhost:8080/tags

# 4. Add tags to the post (assuming post ID 1 and tag IDs 1 and 2)
echo -e "\n\nAdding tags to post..."
curl -X PUT http://localhost:8080/posts/1/tags/1

curl -X PUT http://localhost:8080/posts/1/tags/2

# 5. Get post with ID 1
echo -e "\n\nGetting post by ID..."
curl http://localhost:8080/posts/1

# 6. Get post by slug
echo -e "\n\nGetting post by slug..."
curl http://localhost:8080/posts/slug/my-first-post

# 7. Get tags for the post
echo -e "\n\nGetting tags for post..."
curl http://localhost:8080/posts/1/tags

# 8. Update the post
echo -e "\n\nUpdating post..."
curl -X PATCH http://localhost:8080/posts \
  -H "Content-Type: application/json" \
  -d '{
    "id": 1,
    "title": "My Updated Post",
    "description": "An updated description"
  }'

# 9. List all posts
echo -e "\n\nListing all posts..."
curl "http://localhost:8080/posts?limit=10&offset=0&published_only=true"

# 10. Remove a tag from the post
echo -e "\n\nRemoving tag from post..."
curl -X DELETE http://localhost:8080/posts/1/tags/1

# Optional: Clean up by deleting the post
# echo -e "\n\nDeleting post..."
# curl -X DELETE http://localhost:8080/posts/1
