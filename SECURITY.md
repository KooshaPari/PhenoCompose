# Security Policy — PhenoCompose

PhenoCompose is the policy-composition engine that the rest of the
Phenotype ecosystem relies on for type-safe enforcement of capability
declarations, resource limits, and provenance tags. A compromised
PhenoCompose can issue a forged attestation to any downstream
consumer. Security is therefore a primary feature, not an
afterthought. This document explains how to report a vulnerability,
what to expect from us, and how we handle disclosure.

---

## Supported Versions

PhenoCompose follows [semantic versioning](https://semver.org/). The
following table lists the release lines that currently receive
security updates:

| Version line | Status        | Security fixes                | End-of-life |
| ------------ | ------------- | ----------------------------- | ----------- |
| `2.3.x`      | **Active**    | ✅ Backported                  | TBA         |
| `2.2.x`      | Maintenance   | ✅ Until 2026-09-30            | 2026-09-30  |
| `2.1.x`      | End of life   | ❌ No longer receiving updates | 2025-12-31  |
| `2.0.x`      | End of life   | ❌ No longer receiving updates | 2025-06-30  |
| `1.x`        | End of life   | ❌                             | 2024-12-31  |
| `< 1.0`      | Not supported | ❌                             | n/a         |

> **Recommendation:** always run the latest patch release of the
> latest two minor lines. We will publish a CVE and a GHSA for every
> security fix landed on `main`.

## Reporting a Vulnerability

**Please do not file a public GitHub issue for security bugs.**

The fastest, most private way to report a vulnerability is via one
of the channels below. Choose the one you are most comfortable with:

1. **GitHub private vulnerability reporting** —
   _Repository → Security → Advisories → "New draft security
   advisory"_. This is the preferred channel; it gives you a private
   thread with the maintainers, a CVE assignment, and a coordinated
   disclosure workflow.
2. **Email** — `security@phenotype.internal` (PGP key fingerprint:
   `B5C7 1F2E 9D44 8A6B 7E3C  4F2A 19AB 6C3D 8E1F 0A2B`). The mailbox
   is monitored 24/7 and triaged within 24 hours.
3. **Signal** — `@koosha.42` on Signal. Ask for our Signal safety
   number out-of-band before sharing details.

When you write in, please include (to the extent you can):

- A clear description of the issue and its impact.
- A reproducer — minimal policy fragment, a malformed input that
  triggers the bug, a `phenocompose-verify` invocation that
  accepts a forged attestation, etc.
- The affected commit SHA, tag, or release version.
- Any known workarounds or mitigations.
- Your name / handle for credit (optional; we will not credit by
  default if you request anonymity).

### What _not_ to send

- Do not include real customer data, tokens, or PII in a report.
- Do not exploit the issue beyond what is necessary to demonstrate
  it.
- Do not publish details, screenshots, or PoCs publicly until we
  have agreed a disclosure date (see §4).

## Response Timeline

We commit to the following SLOs. "Business hours" = 09:00–18:00 UTC,
Mon–Fri excluding Phenotype holidays.

| Stage                             | SLO                          |
| --------------------------------- | ---------------------------- |
| **Acknowledgement**               | ≤ 24 hours, every report     |
| **Triage & severity assignment**  | ≤ 3 business days            |
| **Patch for Critical / High**     | ≤ 7 days                     |
| **Patch for Medium**              | ≤ 30 days                    |
| **Patch for Low / Informational** | ≤ 90 days (or accepted-risk) |
| **CVE / GHSA assignment**         | ≤ 24 hours after triage      |
| **Disclosure coordination**       | Per §4                       |

We will keep you informed at every step. If we cannot meet an SLO we
will tell you why, and we will agree a new date with you.

## Coordinated Disclosure

We follow a 90-day coordinated disclosure window from the date of
acknowledgement, modelled on
[Google's project-zero timeline](https://googleprojectzero.blogspot.com/p/vulnerability-disclosure-faq.html).
Concretely:

- **Day 0** — you report the issue.
- **Day 0–7** — we triage, agree severity, and start a fix branch.
- **Day 7–60** — we develop, test, and backport the fix on a private
  advisory branch. For attestation- or solver-bypass issues we also
  rotate the relevant signing key (with a planned rollover window).
- **Day 60–75** — we prepare the advisory, CVE, and release notes.
- **Day 75–90** — embargo; we agree a release date with you and
  downstream consumers.
- **Day 90** — public disclosure: advisory + CVE + release tags +
  blog post. We credit you in the advisory unless you opted out.

We are happy to negotiate the disclosure date, especially for
issues that affect the attestation pipeline (which requires
coordinated key rotation) or that require substantial coordinated
rollout across the Phenotype mesh. Just tell us your constraints.

## Severity Rating

We use CVSS v3.1 base scores as a starting point:

| Severity     | CVSS range | Examples                                         |
| ------------ | ---------- | ------------------------------------------------ |
| **Critical** | 9.0 – 10.0 | Attestation forgery, solver sandbox escape,      |
|              |            | key-rotation bypass, RCE via policy fragment     |
| **High**     | 7.0 – 8.9  | Privilege escalation across composed policies,   |
|              |            | persisted supply-chain backdoor, DoS that        |
|              |            | locks downstream mesh in unsat state             |
| **Medium**   | 4.0 – 6.9  | Information disclosure, partial policy bypass,   |
|              |            | targeted DoS via crafted fragment                |
| **Low**      | 0.1 – 3.9  | Local-only info leaks, hardening recommendations |
| **Info**     | 0.0        | Best-practice deviations, no direct impact       |

## Security Tooling

PhenoCompose is scanned continuously by:

- `cargo audit` + `cargo deny` (RustSec + license).
- `govulncheck` (Go vuln DB, on every PR — for the OPA / Cedar
  adapters).
- `osv-scanner` across lockfiles (Rust, Go, npm, PyPI).
- `pip-audit` for the Python binding.
- GitHub CodeQL (Rust, Go, JavaScript/TypeScript, Python).
- OpenSSF Scorecard (weekly).
- `trivy` filesystem scan in the release pipeline.
- Sigstore `cosign sign` + `cosign verify` for release artifacts.
- `cargo vet` for the supply-chain provenance tree.

Reproduce locally with:

```bash
task security:scan
```

## Out of Scope

The following are **not** considered security vulnerabilities in
PhenoCompose and should be filed as regular bugs:

- Reports about a policy fragment that the operator authored which
  is _too permissive_. That is a usability / linting concern, not a
  security boundary.
- Findings that require physical access to the host running the
  PhenoCompose daemon.
- "Theoretical" issues without a concrete attack path.
- Reports against unsupported (EOL) release lines.

## Bug Bounty

PhenoCompose is not currently running a paid bug bounty programme. We
do publicly credit researchers in the GitHub Security Advisory and in
the release notes, and we are happy to coordinate a joint blog post
with you after disclosure.

## Recognition

We are grateful to the following researchers for responsible
disclosures (most recent first):

- _Awaiting first advisory._

Thank you for helping keep PhenoCompose — and every composition that
depends on it — safe.
