# Portal Access Control - Authentication System

This backend implements a JWT-based authentication system for protecting endpoints.

## Environment Variables

Create a `.env` file in the backend directory with:

```
DATABASE_URL=postgresql://username:password@localhost/database_name
AUTH_PASS=your_secret_password
JWT_SECRET=your_jwt_secret_key
```

## Authentication Flow

### 1. Authenticate to get a token

**Request:**
```bash
POST /authenticate
Content-Type: application/x-www-form-urlencoded

password=your_secret_password
```

**Response (success):**
```json
{
  "message": "Authentication successful",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (failure):**
```
401 Unauthorized
```

### 2. Use the token to access protected endpoints

**Request:**
```bash
GET /protected
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response:**
```json
{
  "message": "This is a protected endpoint",
  "user": "authenticated_user",
  "authenticated": true
}
```

## How to Protect Your Endpoints

To protect any endpoint, simply add the `AuthenticatedUser` parameter to your function:

```rust
#[get("/your-protected-endpoint")]
pub fn your_protected_endpoint(
    _pool_state: &State<Pool<Postgres>>,
    user: AuthenticatedUser  // This automatically validates the JWT token
) -> Result<Json<serde_json::Value>, Status> {
    // Your endpoint logic here
    // Access user info via user.0.sub, user.0.exp, etc.
    Ok(Json(serde_json::json!({"message": "Success"})))
}
```

## Token Expiration

JWT tokens expire after 24 hours by default. You can modify this in `src/auth.rs` in the `Claims::new()` method.

## Security Notes

- The JWT secret should be a strong, random string
- Tokens are signed with HS256 algorithm
- Always use HTTPS in production
- Consider implementing token refresh mechanisms for long-lived sessions
