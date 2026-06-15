// SPDX-License-Identifier: MIT OR Apache-2.0
package domain

import (
	"errors"
	"strings"
	"unicode"
)

// maxSandboxIDLen caps the accepted SandboxID length. Sandbox IDs longer
// than this are rejected by Validate to prevent unbounded values from
// being smuggled into log lines, file paths, or IPC payloads.
const maxSandboxIDLen = 128

// SandboxID is a strongly-typed sandbox identifier. It is intentionally a
// distinct named type (rather than a plain string) so that IDs are not
// silently interchangeable with arbitrary user input or other identifier
// types in the system.
type SandboxID string

// String returns the underlying string representation of the SandboxID.
func (s SandboxID) String() string {
	return string(s)
}

// IsEmpty reports whether the SandboxID has no value. Whitespace-only
// values are NOT considered empty here; use Validate for that check.
func (s SandboxID) IsEmpty() bool {
	return string(s) == ""
}

// ErrInvalidSandboxID is returned by SandboxID.Validate when the ID fails
// to satisfy the format requirements.
var ErrInvalidSandboxID = errors.New("invalid sandbox id")

// Validate checks that the SandboxID is well-formed: non-empty, not
// purely whitespace, and within the maximum length. Allowed characters
// are letters, digits, hyphens, underscores, and dots.
func (s SandboxID) Validate() error {
	v := string(s)
	if v == "" {
		return errors.Join(ErrInvalidSandboxID, errors.New("must not be empty"))
	}
	if len(v) > maxSandboxIDLen {
		return errors.Join(ErrInvalidSandboxID, errors.New("must not exceed max length"))
	}
	if strings.TrimSpace(v) == "" {
		return errors.Join(ErrInvalidSandboxID, errors.New("must not be whitespace only"))
	}
	for _, r := range v {
		if !isAllowedIDRune(r) {
			return errors.Join(ErrInvalidSandboxID, errors.New("contains disallowed characters"))
		}
	}
	return nil
}

// isAllowedIDRune reports whether r is permitted within a SandboxID.
// We accept the common subset used across sandboxes (letters, digits,
// '-', '_', '.') and reject anything else, including spaces and control
// characters.
func isAllowedIDRune(r rune) bool {
	return unicode.IsLetter(r) || unicode.IsDigit(r) || r == '-' || r == '_' || r == '.'
}

// Equal reports whether two SandboxID values are identical.
func (s SandboxID) Equal(other SandboxID) bool {
	return s == other
}

// StartsWith reports whether the SandboxID begins with prefix. By
// convention, an empty prefix does NOT match (mirroring the test
// expectations and avoiding accidental "matches everything" results).
func (s SandboxID) StartsWith(prefix string) bool {
	if prefix == "" {
		return false
	}
	return strings.HasPrefix(string(s), prefix)
}

// Contains reports whether the SandboxID contains the given substring.
func (s SandboxID) Contains(substr string) bool {
	return strings.Contains(string(s), substr)
}

// TrimPrefix removes the leading prefix from the SandboxID and returns
// the remaining string. If the SandboxID does not start with prefix,
// the original string is returned unchanged.
func (s SandboxID) TrimPrefix(prefix string) string {
	return strings.TrimPrefix(string(s), prefix)
}
