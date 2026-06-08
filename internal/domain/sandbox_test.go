package domain

import (
	"strings"
	"testing"
)

func TestSandboxID_String(t *testing.T) {
	tests := []struct {
		name string
		id   SandboxID
		want string
	}{
		{"empty", SandboxID(""), ""},
		{"simple", SandboxID("sb-1"), "sb-1"},
		{"uuid", SandboxID("550e8400-e29b-41d4-a716-446655440000"), "550e8400-e29b-41d4-a716-446655440000"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.id.String(); got != tt.want {
				t.Errorf("SandboxID.String() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestSandboxID_IsEmpty(t *testing.T) {
	if !(SandboxID("")).IsEmpty() {
		t.Error("empty SandboxID should be empty")
	}
	if (SandboxID("sb-1")).IsEmpty() {
		t.Error("non-empty SandboxID should not be empty")
	}
}

func TestSandboxID_Validate(t *testing.T) {
	tests := []struct {
		name    string
		id      SandboxID
		wantErr bool
	}{
		{"empty is invalid", SandboxID(""), true},
		{"valid", SandboxID("sb-1"), false},
		{"valid with hyphens", SandboxID("sb-with-dashes"), false},
		{"valid with underscores", SandboxID("sb_with_underscores"), false},
		{"whitespace only", SandboxID("   "), true},
		{"too long", SandboxID(strings.Repeat("a", 300)), true},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.id.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestSandboxID_Equal(t *testing.T) {
	a := SandboxID("sb-1")
	b := SandboxID("sb-1")
	c := SandboxID("sb-2")
	if !a.Equal(b) {
		t.Error("same IDs should be equal")
	}
	if a.Equal(c) {
		t.Error("different IDs should not be equal")
	}
}

func TestSandboxID_StartsWith(t *testing.T) {
	id := SandboxID("sb-12345")
	if !id.StartsWith("sb-") {
		t.Error("expected to start with sb-")
	}
	if id.StartsWith("xx-") {
		t.Error("should not start with xx-")
	}
	if id.StartsWith("") {
		t.Error("empty prefix should not match by convention")
	}
}

func TestSandboxID_Contains(t *testing.T) {
	id := SandboxID("sb-12345-foo")
	if !id.Contains("12345") {
		t.Error("expected to contain 12345")
	}
	if id.Contains("nope") {
		t.Error("should not contain nope")
	}
}

func TestSandboxID_TrimPrefix(t *testing.T) {
	id := SandboxID("sb-12345")
	got := id.TrimPrefix("sb-")
	if got != "12345" {
		t.Errorf("TrimPrefix = %q, want %q", got, "12345")
	}
}
