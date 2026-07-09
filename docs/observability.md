# PhenoCompose Observability

Audit log + structured events for compliance + debugging.

## Audit log

Every privileged operation must be recorded in an immutable audit log.

### Event types

```rust
#[derive(AuditEvent, Serialize)]
pub enum AuditEvent {
    UserAuthenticated { user_id: String, mfa: bool, ip: IpAddr },
    SecretAccessed { user_id: String, secret_ref: String, op: AccessOp },
    SandboxCreated { user_id: String, sandbox_id: String, image: String },
    ConfigChanged { user_id: String, key: String, old: Value, new: Value },
    AdminAction { user_id: String, action: String, target: String },
}
```

### Append-only storage

```rust
use phenocompose_audit::{AuditLog, AuditEvent};

let log = AuditLog::open("postgres://...").await?;
log.append(&AuditEvent::UserAuthenticated { ... }).await?;
```

WORM semantics: no UPDATE or DELETE allowed at the DB level.

## Rotation

- Daily rotation: `audit_2026-07-09.log` -> `audit_2026-07-10.log`
- Compression: gzip after 7 days
- Retention: 7 years (compliance)
- Cold storage: S3 (Glacier) after 90 days

## Querying

```sql
SELECT event_type, count(*)
FROM audit_log
WHERE created_at > NOW() - INTERVAL '7 days'
GROUP BY event_type;
```

## Retention policies

- Hot (Postgres): 90 days
- Warm (S3 IA): 1 year
- Cold (S3 Glacier): 7 years
- Tamper-evident: hash-chain each entry
