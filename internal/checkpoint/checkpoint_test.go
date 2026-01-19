package checkpoint

import (
	"os"
	"testing"
	"time"
)

func TestSaveLoad(t *testing.T) {
	// Clean up before and after test
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	cp := &Checkpoint{
		TaskID: "test-task-123",
	}

	if err := Save(cp); err != nil {
		t.Fatalf("Save failed: %v", err)
	}

	// Verify defaults were set
	if cp.Version != "1" {
		t.Errorf("Version not defaulted: got %q, want %q", cp.Version, "1")
	}
	if cp.Status != StatusInProgress {
		t.Errorf("Status not defaulted: got %q, want %q", cp.Status, StatusInProgress)
	}
	if cp.StartedAt.IsZero() {
		t.Error("StartedAt not defaulted")
	}
	if cp.LastActivity.IsZero() {
		t.Error("LastActivity not defaulted")
	}

	loaded, err := Load()
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}

	if loaded == nil {
		t.Fatal("Load returned nil")
	}

	if loaded.TaskID != cp.TaskID {
		t.Errorf("TaskID mismatch: got %q, want %q", loaded.TaskID, cp.TaskID)
	}
	if loaded.Version != cp.Version {
		t.Errorf("Version mismatch: got %q, want %q", loaded.Version, cp.Version)
	}
	if loaded.Status != cp.Status {
		t.Errorf("Status mismatch: got %q, want %q", loaded.Status, cp.Status)
	}
}

func TestLoad_NoCheckpoint(t *testing.T) {
	// Clean up to ensure no checkpoint exists
	os.RemoveAll(".arx")

	cp, err := Load()
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}

	if cp != nil {
		t.Errorf("Expected nil checkpoint, got %+v", cp)
	}
}

func TestClear(t *testing.T) {
	// Clean up before test
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create a checkpoint first
	cp := &Checkpoint{
		TaskID: "test-task",
	}
	if err := Save(cp); err != nil {
		t.Fatalf("Save failed: %v", err)
	}

	// Clear it
	if err := Clear(); err != nil {
		t.Fatalf("Clear failed: %v", err)
	}

	// Verify it's gone
	loaded, err := Load()
	if err != nil {
		t.Fatalf("Load after Clear failed: %v", err)
	}
	if loaded != nil {
		t.Errorf("Expected nil after Clear, got %+v", loaded)
	}
}

func TestClear_NoCheckpoint(t *testing.T) {
	// Clean up to ensure no checkpoint exists
	os.RemoveAll(".arx")

	// Clear should not fail if no checkpoint exists
	if err := Clear(); err != nil {
		t.Errorf("Clear failed on non-existent checkpoint: %v", err)
	}
}

func TestIsStale(t *testing.T) {
	testCases := []struct {
		name         string
		lastActivity time.Time
		threshold    time.Duration
		wantStale    bool
	}{
		{
			name:         "recent activity",
			lastActivity: time.Now().Add(-1 * time.Hour),
			threshold:    72 * time.Hour,
			wantStale:    false,
		},
		{
			name:         "activity just under threshold",
			lastActivity: time.Now().Add(-71 * time.Hour),
			threshold:    72 * time.Hour,
			wantStale:    false,
		},
		{
			name:         "activity past threshold",
			lastActivity: time.Now().Add(-73 * time.Hour),
			threshold:    72 * time.Hour,
			wantStale:    true,
		},
		{
			name:         "very old activity",
			lastActivity: time.Now().Add(-7 * 24 * time.Hour),
			threshold:    72 * time.Hour,
			wantStale:    true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			cp := &Checkpoint{
				LastActivity: tc.lastActivity,
			}
			got := IsStaleWithThreshold(cp, tc.threshold)
			if got != tc.wantStale {
				t.Errorf("IsStaleWithThreshold() = %v, want %v", got, tc.wantStale)
			}
		})
	}
}

func TestIsStale_DefaultThreshold(t *testing.T) {
	// Test with default 72-hour threshold
	recentCP := &Checkpoint{
		LastActivity: time.Now().Add(-1 * time.Hour),
	}
	if IsStale(recentCP) {
		t.Error("Recent checkpoint should not be stale")
	}

	staleCP := &Checkpoint{
		LastActivity: time.Now().Add(-100 * time.Hour),
	}
	if !IsStale(staleCP) {
		t.Error("Old checkpoint should be stale")
	}
}

func TestIsStale_NilCheckpoint(t *testing.T) {
	if IsStale(nil) {
		t.Error("Nil checkpoint should not be considered stale")
	}
}
