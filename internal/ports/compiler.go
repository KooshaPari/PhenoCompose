// SPDX-License-Identifier: MIT OR Apache-2.0
// Package ports defines the interfaces (ports) for the hexagonal architecture.
package ports

import "context"

// Compiler defines the interface for image/artifact compilation ports.
// Implementations build a deployable artifact (e.g., a container image
// or a microVM kernel+rootfs bundle) from a source directory or spec
// and tag it so it can be referenced later.
type Compiler interface {
	// Build compiles the source into a tagged artifact and returns a
	// reference (e.g., an OCI image ref) that can be used to start a
	// sandbox.
	Build(ctx context.Context, src, tag string) (string, error)
}
