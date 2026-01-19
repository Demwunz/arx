package journal

import (
	"os"
	"regexp"
	"testing"
	"time"
)

func TestGenerateID(t *testing.T) {
	testCases := []EntryType{
		EntryTypeClarification,
		EntryTypeDecision,
		EntryTypeOverride,
		EntryTypeBlocker,
		EntryTypeAssumption,
		EntryTypeRisk,
		EntryTypeDefer,
		EntryTypeTombstone,
	}

	pattern := regexp.MustCompile(`^[a-z]+-\d{4}-\d{2}-\d{2}-[0-9a-f]{6}$`)

	for _, tc := range testCases {
		t.Run(string(tc), func(t *testing.T) {
			id, err := GenerateID(tc)
			if err != nil {
				t.Fatalf("GenerateID failed: %v", err)
			}

			if !pattern.MatchString(id) {
				t.Errorf("ID %q does not match pattern {type}-{YYYY-MM-DD}-{6-char-hex}", id)
			}

			// Verify type prefix
			expectedPrefix := string(tc) + "-"
			if id[:len(expectedPrefix)] != expectedPrefix {
				t.Errorf("ID %q does not start with expected prefix %q", id, expectedPrefix)
			}
		})
	}
}

func TestGenerateID_Uniqueness(t *testing.T) {
	ids := make(map[string]bool)
	for i := 0; i < 100; i++ {
		id, err := GenerateID(EntryTypeDecision)
		if err != nil {
			t.Fatalf("GenerateID failed: %v", err)
		}
		if ids[id] {
			t.Errorf("Duplicate ID generated: %s", id)
		}
		ids[id] = true
	}
}

func TestWriteReadRoundTrip(t *testing.T) {
	// Setup test directory
	tempDir, err := os.MkdirTemp("", "journal-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Temporarily override journal directory - this is a simplified approach
	// In production, you'd use dependency injection
	testJournalDir := tempDir + "/.arx/journal"

	entry := &Entry{
		ID:      "decision-2026-01-19-abc123",
		Type:    EntryTypeDecision,
		Actor:   ActorHuman,
		Date:    time.Date(2026, 1, 19, 10, 0, 0, 0, time.UTC),
		Title:   "Test Decision",
		Content: "This is the content of the decision.\n\nWith multiple paragraphs.",
	}

	// Create directory and write entry manually for isolated test
	if err := os.MkdirAll(testJournalDir, 0755); err != nil {
		t.Fatalf("Failed to create test journal dir: %v", err)
	}

	// Use Write function (requires setting JournalDir or using the actual dir)
	err = Write(entry)
	if err != nil {
		t.Fatalf("Write failed: %v", err)
	}
	defer os.RemoveAll(".arx") // Clean up test artifacts

	entries, err := ReadAll()
	if err != nil {
		t.Fatalf("ReadAll failed: %v", err)
	}

	if len(entries) != 1 {
		t.Fatalf("Expected 1 entry, got %d", len(entries))
	}

	readEntry := entries[0]

	if readEntry.ID != entry.ID {
		t.Errorf("ID mismatch: got %q, want %q", readEntry.ID, entry.ID)
	}
	if readEntry.Type != entry.Type {
		t.Errorf("Type mismatch: got %q, want %q", readEntry.Type, entry.Type)
	}
	if readEntry.Actor != entry.Actor {
		t.Errorf("Actor mismatch: got %q, want %q", readEntry.Actor, entry.Actor)
	}
	if readEntry.Title != entry.Title {
		t.Errorf("Title mismatch: got %q, want %q", readEntry.Title, entry.Title)
	}
	if readEntry.Content != entry.Content {
		t.Errorf("Content mismatch: got %q, want %q", readEntry.Content, entry.Content)
	}
}

func TestGetState_DerivedState(t *testing.T) {
	// Clean up before test
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create entries with supersedes relationship
	entry1 := &Entry{
		ID:      "decision-2026-01-19-000001",
		Type:    EntryTypeDecision,
		Actor:   ActorHuman,
		Date:    time.Date(2026, 1, 19, 10, 0, 0, 0, time.UTC),
		Title:   "Original Decision",
		Content: "Original content",
	}

	entry2 := &Entry{
		ID:         "decision-2026-01-19-000002",
		Type:       EntryTypeDecision,
		Actor:      ActorHuman,
		Date:       time.Date(2026, 1, 19, 11, 0, 0, 0, time.UTC),
		Title:      "Updated Decision",
		Content:    "Updated content",
		Supersedes: "decision-2026-01-19-000001",
	}

	entry3 := &Entry{
		ID:         "assumption-2026-01-19-000003",
		Type:       EntryTypeAssumption,
		Actor:      ActorPlanner,
		Date:       time.Date(2026, 1, 19, 12, 0, 0, 0, time.UTC),
		Title:      "Reversed Assumption",
		Content:    "This was wrong",
		ReversedBy: "decision-2026-01-19-000004",
	}

	for _, e := range []*Entry{entry1, entry2, entry3} {
		if err := Write(e); err != nil {
			t.Fatalf("Failed to write entry: %v", err)
		}
	}

	state, err := GetState()
	if err != nil {
		t.Fatalf("GetState failed: %v", err)
	}

	// Check entry1 is superseded
	if state.Entries[entry1.ID].State != StateSuperseded {
		t.Errorf("Entry1 should be superseded, got %s", state.Entries[entry1.ID].State)
	}

	// Check entry2 is active
	if state.Entries[entry2.ID].State != StateActive {
		t.Errorf("Entry2 should be active, got %s", state.Entries[entry2.ID].State)
	}

	// Check entry3 is reversed
	if state.Entries[entry3.ID].State != StateReversed {
		t.Errorf("Entry3 should be reversed, got %s", state.Entries[entry3.ID].State)
	}
}
