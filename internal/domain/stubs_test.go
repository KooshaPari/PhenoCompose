package domain

import "testing"

func TestStub_Struct(t *testing.T) {
	// Stub is a placeholder type. Verify the zero value is well-formed.
	s := Stub{}
	if s.Name != "" {
		t.Errorf("zero Stub.Name = %q, want empty", s.Name)
	}
}

func TestStub_WithFields(t *testing.T) {
	s := Stub{
		Name: "test-stub",
	}
	if s.Name != "test-stub" {
		t.Errorf("Stub.Name = %q, want %q", s.Name, "test-stub")
	}
}

func TestStub_ImplementsStringerLike(t *testing.T) {
	// We don't require Stringer, but verify the struct can be used as a value type.
	s1 := Stub{Name: "x"}
	s2 := Stub{Name: "x"}
	if s1 != s2 {
		t.Error("structs with same field values should be equal")
	}
}
