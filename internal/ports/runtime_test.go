package ports

import (
	"context"
	"testing"
)

// fakeRuntime is a test double for the Runtime port.
type fakeRuntime struct {
	startCalls int
	sandboxID  string
	err        error
}

func (f *fakeRuntime) Start(ctx context.Context, image string) (string, error) {
	f.startCalls++
	return f.sandboxID, f.err
}

func TestRuntime_PortSatisfiedByFake(t *testing.T) {
	var _ Runtime = (*fakeRuntime)(nil)
}

func TestRuntime_FakeRecordsStartCalls(t *testing.T) {
	fr := &fakeRuntime{sandboxID: "sb-42"}
	id, err := fr.Start(context.Background(), "img")
	if err != nil {
		t.Fatalf("Start err = %v, want nil", err)
	}
	if id != "sb-42" {
		t.Errorf("sandboxID = %q, want %q", id, "sb-42")
	}
	if fr.startCalls != 1 {
		t.Errorf("startCalls = %d, want 1", fr.startCalls)
	}
}

func TestRuntime_FakePropagatesError(t *testing.T) {
	wantErr := errFake("runtime down")
	fr := &fakeRuntime{err: wantErr}
	_, err := fr.Start(context.Background(), "img")
	if err != wantErr {
		t.Errorf("err = %v, want %v", err, wantErr)
	}
}
