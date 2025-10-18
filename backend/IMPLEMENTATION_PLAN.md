# Rust SSR Web Dashboard - Implementation Plan

> Implementation roadmap for the portal access control backend system

---

## Overview

This document outlines the step-by-step implementation plan for building a Rust SSR Web Dashboard for key management and logs. The plan follows a hierarchical approach, ensuring core functionality is built first, with policies and groups implemented last as requested.

### Current State
- ✅ Basic Rocket setup with PostgreSQL connection
- ✅ Health check endpoint
- ✅ CORS configuration
- ❌ Authentication system
- ❌ Database schema
- ❌ Core models and routes
- ❌ Templates and UI

---

## Phase 1: Foundation & Core Infrastructure

### 1.1 Configuration & Environment Setup
**Priority: Critical | Estimated Time: 2-3 hours**

**Files to Create/Modify:**
- `src/config.rs` - Configuration management
- `.env.example` - Environment template
- `src/db.rs` - Database connection and migrations

**Tasks:**
- [ ] Create configuration struct for app settings
- [ ] Implement Argon2 password hashing utilities
- [ ] Set up environment variable management
- [ ] Create database connection pool management
- [ ] Implement database migration system
- [ ] Add configuration validation

**Dependencies to Add:**
```toml
argon2 = "0.5"
config = "0.14"
migrations = "0.1"
```

### 1.2 Authentication System
**Priority: Critical | Estimated Time: 4-5 hours**

**Files to Create/Modify:**
- `src/auth.rs` - Authentication logic
- `src/middleware.rs` - Auth middleware

**Tasks:**
- [ ] Implement password setup flow (first-time setup)
- [ ] Create login validation logic
- [ ] Implement session management with signed cookies
- [ ] Add session TTL (24 hours) with revalidation
- [ ] Create authentication middleware
- [ ] Add logout functionality
- [ ] Implement session cleanup

**Dependencies to Add:**
```toml
cookie = { version = "0.18", features = ["signed"] }
time = "0.3"
```

---

## Phase 2: Core Models & Data Layer

### 2.1 Database Schema Implementation
**Priority: Critical | Estimated Time: 3-4 hours**

**Migration Files to Create:**
- `migrations/001_create_logs_table.sql`
- `migrations/002_create_keys_table.sql`
- `migrations/003_create_invites_table.sql`
- `migrations/004_create_settings_table.sql`
- `migrations/005_create_policies_table.sql`
- `migrations/006_create_groups_table.sql`

**Tasks:**
- [ ] Create logs table with proper indexes
- [ ] Create keys table with foreign key constraints
- [ ] Create invites table with token management
- [ ] Create settings table for app configuration
- [ ] Create policies table (for later use)
- [ ] Create groups table (for later use)
- [ ] Add proper indexes for performance
- [ ] Set up foreign key relationships

### 2.2 Core Models
**Priority: High | Estimated Time: 4-5 hours**

**Files to Create:**
- `src/models/log.rs`
- `src/models/key.rs`
- `src/models/invite.rs`
- `src/models/settings.rs`

**Tasks:**
- [ ] Implement Log model with CRUD operations
- [ ] Implement Key model with status management
- [ ] Implement Invite model with token generation
- [ ] Implement Settings model for app configuration
- [ ] Add model validation and error handling
- [ ] Implement database query helpers
- [ ] Add model serialization/deserialization

**Dependencies to Add:**
```toml
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 2.3 Utility Functions
**Priority: Medium | Estimated Time: 2-3 hours**

**Files to Create:**
- `src/utils.rs`

**Tasks:**
- [ ] Add date/time formatting utilities
- [ ] Implement NIP-05 profile fetching
- [ ] Add input validation helpers
- [ ] Create error handling utilities
- [ ] Add logging utilities
- [ ] Implement CSV/JSON export helpers

---

## Phase 3: Core Routes & Pages

### 3.1 Authentication Routes
**Priority: Critical | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/routes/login.rs`
- `src/templates/login.hbs`

**Tasks:**
- [ ] Implement GET `/login` - Login form
- [ ] Implement POST `/login` - Password validation
- [ ] Implement GET `/logout` - Session termination
- [ ] Create login template with error handling
- [ ] Add setup mode for first-time password creation
- [ ] Implement redirect logic after authentication

### 3.2 Logs System (Primary Feature)
**Priority: Critical | Estimated Time: 5-6 hours**

**Files to Create:**
- `src/routes/logs.rs`
- `src/sse.rs`
- `src/templates/logs.hbs`
- `src/templates/log_detail.hbs`

**Tasks:**
- [ ] Implement GET `/logs` - Main logs page (landing page)
- [ ] Implement GET `/logs/:id` - Individual log detail
- [ ] Implement GET `/logs/stream` - SSE endpoint
- [ ] Create logs listing template with pagination
- [ ] Create log detail template
- [ ] Implement SSE streaming for live updates
- [ ] Add export functionality (CSV/JSON)
- [ ] Implement infinite scroll or pagination
- [ ] Add live mode toggle

### 3.3 Key Management
**Priority: High | Estimated Time: 4-5 hours**

**Files to Create:**
- `src/routes/keys.rs`
- `src/templates/keys.hbs`
- `src/templates/keys_add.hbs`

**Tasks:**
- [ ] Implement GET `/keys` - Key listing
- [ ] Implement GET `/keys/add` - Add key form
- [ ] Implement POST `/keys/add` - Create key
- [ ] Implement POST `/keys/:id/toggle` - Enable/Disable
- [ ] Implement POST `/keys/:id/delete` - Delete key
- [ ] Create key listing template
- [ ] Create add key form template
- [ ] Implement NIP-05 profile fetching
- [ ] Add key status management
- [ ] Implement expiry date handling

### 3.4 Invitation System
**Priority: High | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/routes/invites.rs`
- `src/templates/invites.hbs`

**Tasks:**
- [ ] Implement GET `/invites` - Invite listing
- [ ] Implement POST `/invites/new` - Create invite
- [ ] Implement invite token generation
- [ ] Add invite expiry management
- [ ] Implement usage tracking
- [ ] Create invite management template
- [ ] Add copy link functionality
- [ ] Implement invite disable/enable

---

## Phase 4: Templates & UI

### 4.1 Base Template System
**Priority: High | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/templates/layout.hbs`
- `static/css/style.css`

**Tasks:**
- [ ] Create base layout template
- [ ] Implement navigation system
- [ ] Add dark-first CSS styling
- [ ] Create responsive design
- [ ] Add error page templates (401, 404, 500)
- [ ] Implement consistent styling across pages
- [ ] Add loading states and transitions

### 4.2 Settings & Configuration
**Priority: Medium | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/routes/settings.rs`
- `src/templates/settings.hbs`

**Tasks:**
- [ ] Implement GET `/settings` - Settings form
- [ ] Implement POST `/settings` - Save settings
- [ ] Add password change functionality
- [ ] Implement theme toggle (dark/light)
- [ ] Add log retention settings
- [ ] Implement default policy settings
- [ ] Create settings template with sections
- [ ] Add settings validation

---

## Phase 5: Advanced Features

### 5.1 Minimal JavaScript
**Priority: Medium | Estimated Time: 2-3 hours**

**Files to Create:**
- `static/js/app.js`

**Tasks:**
- [ ] Implement SSE client for live logs
- [ ] Add copy-to-clipboard functionality
- [ ] Implement theme toggle
- [ ] Add modal interactions
- [ ] Implement live mode toggle
- [ ] Add fade-in animations for new log entries
- [ ] Create minimal JavaScript framework

### 5.2 Error Handling & Edge Cases
**Priority: Medium | Estimated Time: 2-3 hours**

**Tasks:**
- [ ] Implement comprehensive error handling
- [ ] Add input validation and sanitization
- [ ] Create custom error pages
- [ ] Add request logging middleware
- [ ] Implement graceful error recovery
- [ ] Add error reporting and logging

---

## Phase 6: Policy System (End Phase)

### 6.1 Policy Models & Logic
**Priority: Low | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/models/policy.rs`

**Tasks:**
- [ ] Implement Policy model with CRUD operations
- [ ] Add policy validation logic
- [ ] Implement time window handling
- [ ] Add day-of-week management
- [ ] Create policy enforcement logic
- [ ] Add policy expiry handling
- [ ] Implement policy-key relationship

### 6.2 Policy Routes & UI
**Priority: Low | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/routes/policies.rs`
- `src/templates/policies.hbs`
- `src/templates/policies_new.hbs`

**Tasks:**
- [ ] Implement GET `/policies` - Policy listing
- [ ] Implement GET `/policies/new` - Add policy form
- [ ] Implement POST `/policies/new` - Create policy
- [ ] Add policy editing functionality
- [ ] Implement policy deletion
- [ ] Create policy management templates
- [ ] Add policy assignment to keys
- [ ] Implement policy validation UI

---

## Phase 7: Group System (Final Phase)

### 7.1 Group Models & Logic
**Priority: Low | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/models/group.rs`

**Tasks:**
- [ ] Implement Group model with CRUD operations
- [ ] Add group-key relationship management
- [ ] Implement group assignment logic
- [ ] Add group status management
- [ ] Create group validation logic
- [ ] Implement group statistics tracking

### 7.2 Group Routes & UI
**Priority: Low | Estimated Time: 3-4 hours**

**Files to Create:**
- `src/routes/groups.rs`
- `src/templates/groups.hbs`
- `src/templates/groups_view.hbs`

**Tasks:**
- [ ] Implement GET `/groups` - Group listing
- [ ] Implement GET `/groups/:id` - Manage group
- [ ] Implement POST `/groups/new` - Create group
- [ ] Implement POST `/groups/:id/add-key` - Assign key
- [ ] Add group management interface
- [ ] Create group assignment UI
- [ ] Implement group statistics display
- [ ] Add group deletion functionality

---

## Implementation Timeline

| Phase | Duration | Dependencies | Critical Path |
|-------|----------|--------------|---------------|
| Phase 1 | 6-8 hours | None | ✅ Critical |
| Phase 2 | 9-12 hours | Phase 1 | ✅ Critical |
| Phase 3 | 15-19 hours | Phase 2 | ✅ Critical |
| Phase 4 | 6-8 hours | Phase 3 | ⚠️ Important |
| Phase 5 | 4-6 hours | Phase 4 | ⚠️ Important |
| Phase 6 | 6-8 hours | Phase 5 | 🔵 Optional |
| Phase 7 | 6-8 hours | Phase 6 | 🔵 Optional |

**Total Estimated Time: 52-69 hours**

---

## Testing Strategy

### Unit Tests
- [ ] Model validation tests
- [ ] Authentication logic tests
- [ ] Utility function tests
- [ ] Database operation tests

### Integration Tests
- [ ] Route endpoint tests
- [ ] Database integration tests
- [ ] SSE streaming tests
- [ ] Authentication flow tests

### Manual Testing
- [ ] UI/UX testing across browsers
- [ ] Performance testing with large datasets
- [ ] Security testing (session management)
- [ ] Error handling validation

---

## Deployment Considerations

### Environment Setup
- [ ] Docker containerization
- [ ] Environment variable configuration
- [ ] Database migration automation
- [ ] SSL/TLS configuration

### Monitoring & Logging
- [ ] Application logging setup
- [ ] Error monitoring
- [ ] Performance metrics
- [ ] Health check endpoints

### Security
- [ ] Password hashing validation
- [ ] Session security review
- [ ] Input sanitization verification
- [ ] CORS configuration review

---

## Success Criteria

### Phase 1-3 (Core System)
- ✅ User can authenticate with single password
- ✅ Logs page loads as landing page
- ✅ Keys can be added, managed, and deleted
- ✅ Invitations can be created and managed
- ✅ Basic UI is functional and responsive

### Phase 4-5 (Enhanced UI)
- ✅ Settings can be configured
- ✅ Live logs updates work via SSE
- ✅ Theme toggle functions properly
- ✅ Export functionality works

### Phase 6-7 (Advanced Features)
- ✅ Policies can be created and assigned to keys
- ✅ Groups can be managed and keys assigned
- ✅ Full system integration works end-to-end

---

## Notes

- **Technology Stack**: Continuing with Rocket framework for consistency
- **Database**: PostgreSQL with sqlx for async operations
- **Templates**: Handlebars for SSR rendering
- **Security**: Argon2 for password hashing, signed cookies for sessions
- **Real-time**: Server-Sent Events for live log updates
- **Styling**: Minimal CSS with dark-first approach

This implementation plan ensures a systematic approach to building the portal access control system, with core functionality prioritized and policies/groups implemented last as requested.
