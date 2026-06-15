// SPDX-License-Identifier: MIT OR Apache-2.0
// Package ports defines the interfaces (ports) for the hexagonal architecture.
package ports

import "context"

// Runtime defines the interface for lightweight sandbox/OCI runtimes
// (containerd, crun, runc, etc.) that can launch an instance from an
// already-built image and return its identifier.
type Runtime interface {
	// Start launches a sandbox from the given image reference and
	// returns the newly created sandbox ID.
	Start(ctx context.Context, image string) (string, error)
}
