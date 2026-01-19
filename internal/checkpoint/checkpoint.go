package checkpoint

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"
)

const (
	CheckpointPath       = ".arx/checkpoint.json"
	DefaultStaleThreshold = 72 * time.Hour
)

// Status represents the checkpoint status
type Status string

const (
	StatusInProgress Status = "in_progress"
	StatusCompleted  Status = "completed"
	StatusFailed     Status = "failed"
	StatusPaused     Status = "paused"
)

// Checkpoint represents the current task state
type Checkpoint struct {
	Version      string    `json:"version"`
	TaskID       string    `json:"task_id"`
	StartedAt    time.Time `json:"started_at"`
	LastActivity time.Time `json:"last_activity"`
	Status       Status    `json:"status"`
}

// Save writes the checkpoint to disk with auto-initialization of defaults
func Save(cp *Checkpoint) error {
	if cp.Version == "" {
		cp.Version = "1"
	}
	if cp.StartedAt.IsZero() {
		cp.StartedAt = time.Now()
	}
	if cp.LastActivity.IsZero() {
		cp.LastActivity = time.Now()
	}
	if cp.Status == "" {
		cp.Status = StatusInProgress
	}

	dir := filepath.Dir(CheckpointPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create checkpoint directory: %w", err)
	}

	data, err := json.MarshalIndent(cp, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal checkpoint: %w", err)
	}

	if err := os.WriteFile(CheckpointPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write checkpoint file: %w", err)
	}

	return nil
}

// Load reads the checkpoint from disk, returning nil if no checkpoint exists
func Load() (*Checkpoint, error) {
	data, err := os.ReadFile(CheckpointPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil
		}
		return nil, fmt.Errorf("failed to read checkpoint file: %w", err)
	}

	cp := &Checkpoint{}
	if err := json.Unmarshal(data, cp); err != nil {
		return nil, fmt.Errorf("failed to unmarshal checkpoint: %w", err)
	}

	return cp, nil
}

// Clear removes the checkpoint file
func Clear() error {
	err := os.Remove(CheckpointPath)
	if err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to remove checkpoint file: %w", err)
	}
	return nil
}

// IsStale returns true if the checkpoint's last activity exceeds the threshold
func IsStale(cp *Checkpoint) bool {
	return IsStaleWithThreshold(cp, DefaultStaleThreshold)
}

// IsStaleWithThreshold returns true if the checkpoint's last activity exceeds the given threshold
func IsStaleWithThreshold(cp *Checkpoint, threshold time.Duration) bool {
	if cp == nil {
		return false
	}
	return time.Since(cp.LastActivity) > threshold
}
