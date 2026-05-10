package fastcdc

import (
	"testing"
)

func TestFastCDC(t *testing.T) {
	data := make([]byte, 1024*1024) // 1MB
	config := DefaultConfig()
	cdc := NewFastCDC(data, config)

	count := 0
	for {
		chunk, err := cdc.Next()
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if chunk == nil {
			break
		}
		count++
	}

	if count == 0 {
		t.Errorf("expected at least one chunk, got 0")
	}
}
