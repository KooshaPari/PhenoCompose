package ports

import (
	"context"
	"testing"
)

// fakeCompiler is a test double for the Compiler port that records calls
// and returns canned values. Implements the Compiler port from compiler.go.
type fakeCompiler struct {
	buildCalls int
	image      string
	err        error
}

func (f *fakeCompiler) Build(ctx context.Context, src, tag string) (string, error) {
	f.buildCalls++
	return f.image, f.err
}

// TestCompiler_PortSatisfiedByFake verifies that fakeCompiler satisfies the
// Compiler port interface (compile-time assertion).
func TestCompiler_PortSatisfiedByFake(t *testing.T) {
	var _ Compiler = (*fakeCompiler)(nil)
}

func TestCompiler_FakeRecordsBuildCalls(t *testing.T) {
	fc := &fakeCompiler{image: "img-1"}
	ctx := context.Background()
	img, err := fc.Build(ctx, "src", "v1")
	if err != nil {
		t.Fatalf("Build err = %v, want nil", err)
	}
	if img != "img-1" {
		t.Errorf("Build image = %q, want %q", img, "img-1")
	}
	if fc.buildCalls != 1 {
		t.Errorf("buildCalls = %d, want 1", fc.buildCalls)
	}
}

func TestCompiler_FakePropagatesError(t *testing.T) {
	wantErr := errFake("compile failed")
	fc := &fakeCompiler{err: wantErr}
	_, err := fc.Build(context.Background(), "src", "v1")
	if err != wantErr {
		t.Errorf("err = %v, want %v", err, wantErr)
	}
}

type errFake string

func (e errFake) Error() string { return string(e) }
