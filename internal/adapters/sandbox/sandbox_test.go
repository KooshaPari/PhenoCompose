// SPDX-License-Identifier: MIT OR Apache-2.0
package sandbox

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/kooshapari/phenocompose/internal/domain"
)

func TestAdapter_Create(t *testing.T) {
	ctx := context.Background()
	a := NewAdapter()

	config := domain.SandboxConfig{
		Name:   "test-sandbox",
		VMType: domain.VMFlavorLima,
	}

	before := time.Now()
	sandbox, err := a.Create(ctx, config)
	after := time.Now()
	if err != nil {
		t.Fatalf("Create returned unexpected error: %v", err)
	}
	if sandbox == nil {
		t.Fatal("Create returned nil sandbox with no error")
	}
	if sandbox.ID == "" {
		t.Error("Create returned sandbox with empty ID")
	}
	if !strings.HasPrefix(sandbox.ID, "sandbox-") {
		t.Errorf("Create sandbox ID = %q, want prefix %q", sandbox.ID, "sandbox-")
	}
	if sandbox.Name != config.Name {
		t.Errorf("Create sandbox Name = %q, want %q", sandbox.Name, config.Name)
	}
	if sandbox.Status != domain.SandboxStatusPending {
		t.Errorf("Create sandbox Status = %q, want %q", sandbox.Status, domain.SandboxStatusPending)
	}
	if sandbox.VMFlavor != config.VMType {
		t.Errorf("Create sandbox VMFlavor = %q, want %q", sandbox.VMFlavor, config.VMType)
	}
	if sandbox.CreatedAt.Before(before) || sandbox.CreatedAt.After(after) {
		t.Errorf("Create sandbox CreatedAt = %v, want between %v and %v", sandbox.CreatedAt, before, after)
	}

	// The sandbox must be retrievable via Start (verifies it was stored in the adapter).
	if err := a.Start(ctx, sandbox.ID); err != nil {
		t.Errorf("Start on just-created sandbox returned error: %v", err)
	}

	// A second call should produce a different ID.
	other, err := a.Create(ctx, domain.SandboxConfig{Name: "other"})
	if err != nil {
		t.Fatalf("second Create returned unexpected error: %v", err)
	}
	if other.ID == sandbox.ID {
		t.Errorf("Create returned duplicate ID %q on second call", other.ID)
	}
}
